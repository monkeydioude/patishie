#![allow(async_fn_in_trait)]
use std::{sync::Arc, time::Duration};

use api::api::{healthcheck, lezgong};
use chrono::Utc;
use config::Settings;
use futures::future::join_all;
use rocket::{launch, routes};
use services::channel::fetch_ready_channels;
use task::spawn_tasks;
use tokio::spawn;
use tokio::time::sleep;
use utils::{DBBag, Second};

pub mod api;
pub mod config;
pub mod converters;
pub mod db;
pub mod entities;
pub mod error;
pub mod services;
pub mod task;
pub mod utils;

fn find_index(ledger: &Vec<i32>, value: &i32) -> Option<usize> {
    let mut res: usize = 0;
    for v in ledger.to_vec().iter() {
        if v == value {
            return Some(res);
        }
        res += 1;
    }

    None
}

#[launch]
async fn launch() -> _ {
    let settings = Arc::new(Settings::new().unwrap());
    let db_handle = Arc::new(db::mongo::get_handle(&settings).await);
    let db_bag = Arc::new(DBBag::new(db_handle.clone()).unwrap());
    let sleep_duration = Second(20).msec();
    let mut ledger = Vec::<i32>::new();

    // spawn(async move { lezgong(routes![healthcheck], 8085).await });
    let _ = spawn(async move {
        loop {
            let channels = fetch_ready_channels(&db_bag.channels_coll).await;
            if channels.is_empty() {
                eprintln!(
                    "({}) Didnt find any channel to refresh. Sleeping for {}",
                    Utc::now().timestamp_millis(),
                    sleep_duration
                );
                sleep(Duration::from_millis(sleep_duration.into())).await;
                continue;
            }

            let mut _tasks = spawn_tasks(&channels, &settings, &db_bag, &mut ledger);
            join_all(_tasks).await;
            eprintln!(
                "({}) Done :) Sleeping for {}ms",
                Utc::now().timestamp_millis(),
                sleep_duration + 1000
            );
            sleep(Duration::from_millis(sleep_duration + 1000)).await;
        }
    });

    lezgong(routes![healthcheck], 8085).await
}
