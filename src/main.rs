#![allow(async_fn_in_trait)]

use std::{sync::Arc, time::Duration, vec::Drain};

use chrono::Utc;
use config::Settings;
use db::{channel::Channels, entities::Timer, items::Items, model::{BlankCollection, CollectionModel}, mongo::Handle, timers::Timers};
use entities::{channel::Channel, potential_articles::PotentialArticle};
use error::Error;
use services::{bakery::get_cookies_from_bakery, channel::get_shortest_sleep, panya::process_data};
use tokio::{spawn, task::JoinHandle, time::sleep};

use crate::db::pipeline::Pipeline;

pub mod config;
pub mod db;
pub mod entities;
pub mod error;
pub mod services;
pub mod utils;
pub mod converters;

struct DBBag {
    // db_handle: Arc<Handle>,
    channels_coll: Channels<Channel>,
    items_coll: Items<PotentialArticle>,
    timers_coll: BlankCollection<Timer>,
} 

impl DBBag {
    pub fn new(db_handle: Arc<Handle>) -> Result<Self, Error> {
        Ok(Self {
            // db_handle: db_handle.clone(),
            channels_coll: Channels::<Channel>::new(db_handle.clone(), "panya")?,
            items_coll: Items::<PotentialArticle>::new(db_handle.clone(), "panya")?,
            timers_coll: Timers::new(db_handle, "panya", "timers")?,
        })
    }
}

async fn update_from_bakery(
    db_bag: Arc<DBBag>,
    settings: Arc<Settings>,
    channel_id: i32,
    channel_name: String
) -> Result<i64, Error> {
    let raw_url = "https://".to_string() + &channel_name;
    let now_before_refresh = Utc::now();
    let parsed_from_bakery = get_cookies_from_bakery(&settings.api_path, &raw_url)
        .await
        .unwrap_or_default();
    let refresh_time = (Utc::now() - now_before_refresh).num_milliseconds();
    let mut success = true;
    if parsed_from_bakery.is_empty() {
        success = false;
        println!("bakery::get_cookies_from_bakery, channel_name: {}, channel_id: {} - no articles found", channel_name, channel_id);
    } else {
        db_bag.timers_coll
            .insert_one(&channel_name, refresh_time)
            .await;
        let _ = process_data(
            &parsed_from_bakery,
            &db_bag.items_coll,
            &db_bag.channels_coll,
            &channel_name,
        );
    }
    db_bag.channels_coll
        .update_refresh_now(channel_id, &*channel_name, success)
        .await
        .ok_or_else(|| error::Error(format!("could not refresh channel id {}, {}", channel_id, channel_name)))
}

async fn find_earliest_refresh(tasks: Drain<'_, JoinHandle<Result<i64, error::Error>>>) -> i64 {
    let refresh = Utc::now().timestamp_millis();
    let mut res: Option<i64> = None;
    for task in tasks {
        let tmp_refresh = match task.await {
            Ok(Ok(t)) => t,
            Ok(Err(err)) => {
                eprintln!("{}", err);
                refresh
            }
            Err(err) => {
                eprintln!("{}", err);
                continue;
            },
        };
        if res.is_none() || tmp_refresh < res.unwrap() {
            res = Some(tmp_refresh);
        }
    }
    res.unwrap_or(refresh)
}

async fn fetch_ready_channels(channels_coll: &Channels<Channel>) -> Vec<Channel> {
    channels_coll
        .find_aggregate(
            Pipeline::single_add_lt(
                "next_refresh", 
                &["last_refresh", "refresh_frequency"], 
                &Utc::now().timestamp_millis()
            )
        )
        .await
        .unwrap_or_default()
}

fn spawn_tasks(
    channels: &Vec<Channel>,
    settings: &Arc<Settings>,
    db_bag: &Arc<DBBag>,
) -> Vec<JoinHandle<Result<i64, Error>>> {
    let mut tasks = vec![];

    for c in channels {
        let channel_name = c.name.clone();
        let channel_id = c.id;
        let db_bag_clone = Arc::clone(&db_bag);
        let settings_clone = Arc::clone(&settings);
        tasks.push(spawn(async move {
            println!("Starting request to {}", channel_name);
            let res = update_from_bakery(
                db_bag_clone,
                settings_clone,
                channel_id,
                channel_name.clone(),
            ).await;
            println!("Done for {}", channel_name);
            res
        }));
    }

    tasks
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Arc::new(Settings::new().unwrap());
    let db_handle = Arc::new(db::mongo::get_handle(&settings).await);
    let db_bag = Arc::new(DBBag::new(db_handle.clone())?);

    loop {
        let channels = fetch_ready_channels(&db_bag.channels_coll).await;
        if channels.is_empty() {
            let sleep_duration = get_shortest_sleep(None, &channels)
                .unwrap_or(settings.default_main_sleep);
            println!("Didnt find any channel to refresh. Sleeping for {}ms", sleep_duration);
            sleep(Duration::from_millis(sleep_duration)).await;
            continue;
        }

        let mut tasks = spawn_tasks(&channels, &settings, &db_bag);
        let refresh = find_earliest_refresh(tasks.drain(..)).await;
        let all_channels = db_bag.channels_coll.find_all()
            .await
            .unwrap_or_default();
        let sleep_duration = get_shortest_sleep(Some(refresh), &all_channels).unwrap_or(settings.default_main_sleep);
        println!("All done! Sleeping for {}ms", sleep_duration);
        sleep(Duration::from_millis(sleep_duration + 1000)).await;
    }
}
