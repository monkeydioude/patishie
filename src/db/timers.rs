use chrono::Utc;

use crate::db::{
    entities::Timer,
    model::{BlankCollection, CollectionModel},
};

pub type Timers<'a> = BlankCollection<Timer>;

impl<'a> Timers<'a> {
    pub async fn insert_one(&self, channel: &str, time_to_refresh: i64) -> Option<()> {
        self.insert_many(&[Timer {
            channel: channel.to_string(),
            update_date: Utc::now().timestamp(),
            time_to_refresh,
        }])
        .await
        .map_err(|err| {
            eprintln!("could not insert in timers collection: {}", err);
            err
        })
        .ok()
        .and(Some(()))
    }
}
