use std::sync::Arc;

use chrono::Utc;
use tokio::{spawn, task::JoinHandle};
use uuid::Uuid;

use crate::{
    config::Settings,
    entities::{channel::Channel, source_type::SourceType},
    error::{self, Error},
    find_index,
    services::{bakery::get_cookies_from_bakery, panya::process_data, rss::get_cookies_from_rss},
    DBBag,
};

async fn update_channel(
    db_bag: Arc<DBBag>,
    settings: Arc<Settings>,
    channel_id: i32,
    channel_name: String,
    channel_url: String,
    log_id: Uuid,
    source_type: SourceType,
) -> Result<i64, Error> {
    // now time
    let now_before_refresh = Utc::now();
    let _ = db_bag
        .channels_coll
        .update_refresh_now(channel_id, &*channel_name, false)
        .await
        .ok_or_else(|| {
            error::Error(format!(
                "could not refresh channel id {}, {}",
                channel_id, channel_name
            ))
        })?;
    // parse result from bakery or rss source
    let parsed_result = match source_type {
        SourceType::RSSFeed => get_cookies_from_rss(&channel_url, channel_id, log_id)
            .await
            .unwrap_or_default(),
        SourceType::Bakery => get_cookies_from_bakery(&settings.api_path, &channel_url, log_id)
            .await
            .unwrap_or_default(),
        SourceType::Other => {
            return Err(Error("SourceType::Other not implemeented yet".to_string()));
        }
    };
    let refresh_time = (Utc::now() - now_before_refresh).num_milliseconds();
    let mut success = true;
    if parsed_result.is_empty() {
        success = false;
        println!(
            "[{}] ({}) source_type: {}, channel_name: {}, channel_id: {} - no articles found",
            log_id,
            Utc::now().timestamp_millis(),
            source_type,
            channel_name,
            channel_id
        );
    } else {
        db_bag
            .timers_coll
            .insert_one(&channel_name, refresh_time)
            .await;
        let res = process_data(
            &parsed_result,
            &db_bag.items_coll,
            &db_bag.channels_coll,
            &channel_name,
            &channel_url,
            source_type,
        )
        .await;
        if res.is_err() {
            println!("[ERR ] {:?}", res.err());
        }
    }
    db_bag
        .channels_coll
        .update_refresh_now(channel_id, &*channel_name, success)
        .await
        .ok_or_else(|| {
            error::Error(format!(
                "could not refresh channel id {}, {}",
                channel_id, channel_name
            ))
        })
}

pub fn spawn_tasks(
    channels: &Vec<Channel>,
    settings: &Arc<Settings>,
    db_bag: &Arc<DBBag>,
    ledger: &mut Vec<i32>,
) -> Vec<JoinHandle<Result<i64, Error>>> {
    let mut tasks = vec![];
    for c in channels {
        let channel_name = c.name.clone();
        let channel_url = c.url.clone();
        let channel_id = c.id;
        let db_bag_clone = Arc::clone(&db_bag);
        let settings_clone = Arc::clone(&settings);
        let source_type = c.source_type.clone();
        if ledger.contains(&channel_id) {
            continue;
        }
        if let Some(v) = find_index(ledger, &channel_id) {
            ledger.remove(v);
        }
        // ledger
        tasks.push(spawn(async move {
            let before = Utc::now();
            let task_id = Uuid::new_v4();
            println!(
                "[{}] ({}) Starting request to {}",
                task_id,
                before.timestamp_millis(),
                channel_name
            );
            let res = update_channel(
                db_bag_clone,
                settings_clone,
                channel_id,
                channel_name.clone(),
                channel_url.clone(),
                task_id,
                source_type,
            )
            .await;
            let after = Utc::now();
            println!(
                "[{}] ({}) Done for {}, in {}ms",
                task_id,
                after.timestamp_millis(),
                channel_url,
                after.timestamp_millis() - before.timestamp_millis()
            );
            res
        }));
    }

    tasks
}
