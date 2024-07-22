#![allow(async_fn_in_trait)]

use std::{sync::Arc, time::Duration};

use chrono::Utc;
use config::Settings;
use services::channel::fetch_ready_channels;
use task::spawn_tasks;
use tokio::time::sleep;
use utils::{DBBag, Second};

pub mod config;
pub mod db;
pub mod entities;
pub mod error;
pub mod services;
pub mod utils;
pub mod converters;
pub mod task;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Arc::new(Settings::new().unwrap());
    let db_handle = Arc::new(db::mongo::get_handle(&settings).await);
    let db_bag = Arc::new(DBBag::new(db_handle.clone())?);
    let sleep_duration = Second(20).msec();
    let mut ledger = Vec::<i32>::new();

    loop {
        let channels = fetch_ready_channels(&db_bag.channels_coll).await;
        if channels.is_empty() {
            println!("({}) Didnt find any channel to refresh. Sleeping for {}", Utc::now().timestamp_millis(), sleep_duration);
            sleep(Duration::from_millis(sleep_duration.into())).await; 
            continue;
        }

        let mut _tasks = spawn_tasks(&channels, &settings, &db_bag, &mut ledger);
        sleep(Duration::from_millis(sleep_duration + 1000)).await;
    }
}
