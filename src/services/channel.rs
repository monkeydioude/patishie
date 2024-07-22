use chrono::Utc;

use crate::{db::{channel::Channels, model::CollectionModel, pipeline::Pipeline}, entities::channel::Channel};

impl Eq for &Channel {}

impl Ord for &Channel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.last_refresh.cmp(&self.last_refresh)
    }
}

pub fn get_shortest_sleep(
    refresh_time: Option<i64>,
    channels: &Vec<Channel>,
) -> Option<u64> {
    if channels.is_empty() {
        return None
    }
    let now_ms = Utc::now().timestamp_millis();
    channels
        .iter()
        .min()
        .and_then(|el| {
            let res = refresh_time.unwrap_or(el.last_refresh) + (el.refresh_frequency as i64) - now_ms;
            if res < 0 {
                return None;
            }
            Some(res as u64)
        })
}

pub async fn fetch_ready_channels(channels_coll: &Channels<Channel>) -> Vec<Channel> {
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