#![allow(async_fn_in_trait)]

use std::{sync::Arc, time::Duration, vec::Drain};

use chrono::Utc;
use config::Settings;
use db::{channel::Channels, items::Items, model::CollectionModel, mongo::Handle, timers::Timers};
use entities::{channel::Channel, potential_articles::PotentialArticle};
use error::Error;
use services::{bakery::get_cookies_from_bakery, channel::get_shortest_sleep, panya::process_data_and_fetch_items};
use tokio::{spawn, task::JoinHandle, time::sleep};

use crate::db::pipeline::Pipeline;

pub mod config;
pub mod db;
pub mod entities;
pub mod error;
pub mod services;
pub mod utils;
pub mod converters;

async fn update_from_bakery(
    db_handle: Arc<Handle>,
    settings: Arc<Settings>,
    channel_id: i32,
    channel_name: String
) -> Result<i64, Error> {
    let raw_url = "https://".to_string() + &channel_name;
    let channels_coll = Channels::<Channel>::new(&db_handle, "panya")?;
    let items_coll = Items::<PotentialArticle>::new(&db_handle, "panya")?;
    let timers_coll = Timers::new(&db_handle, "panya", "timers")?;
    let now_before_refresh = Utc::now();
    let parsed_from_bakery = get_cookies_from_bakery(&settings.api_path, &raw_url)
        .await
        .unwrap_or_default();
    let refresh_time = (Utc::now() - now_before_refresh).num_milliseconds();
    if parsed_from_bakery.is_empty() {
        return Err(error::Error("bakery::get_cookies_from_bakery - no articles found".to_string()));
    }
    timers_coll
        .insert_one(&channel_name, refresh_time)
        .await;
    process_data_and_fetch_items(
        &parsed_from_bakery,
        items_coll,
        &channels_coll,
        &channel_name,
        settings.default_item_per_feed,
    ).await;
    channels_coll
        .update_refresh(channel_id, &*channel_name)
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Arc::new(Settings::new().unwrap());
    let db_handle = Arc::new(db::mongo::get_handle(&settings).await);
    let channel_coll: Channels<Channel> = Channels::new(&db_handle, "panya").unwrap();
    let mut tasks = vec![];

    loop {
        let channels = channel_coll
            .find_aggregate(
                Pipeline::single_add_lt(
                    "next_refresh", 
                    &["last_refresh", "refresh_frequency"], 
                    &Utc::now().timestamp_millis()
                )
            )
            .await
            .unwrap_or_default();
        if channels.is_empty() {
            let sleep_duration = get_shortest_sleep(None, &channels)
                .unwrap_or(settings.default_main_sleep);
            println!("Didnt find any channel to refresh. Sleeping for {}ms", sleep_duration);
            sleep(Duration::from_millis(sleep_duration)).await;
            continue;
        }
        
        for c in &channels {
            let channel_name = c.name.clone();
            let channel_id = c.id;
            let db_handle_clone = Arc::clone(&db_handle);
            let settings_clone = Arc::clone(&settings);
            tasks.push(spawn(async move {
                let res = update_from_bakery(db_handle_clone, settings_clone, channel_id, channel_name.clone()).await;
                res
            }));
        }
        let refresh = find_earliest_refresh(tasks.drain(..)).await;
        let sleep_duration = get_shortest_sleep(Some(refresh), &channels)
            .unwrap_or(settings.default_main_sleep);
        println!("All done! Sleeping for {}ms", sleep_duration);
        sleep(Duration::from_millis(sleep_duration)).await;
    }
}
