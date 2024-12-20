use serde::{Deserialize, Serialize};

use crate::{
    db::{
        channel::Channels,
        model::{CollectionModel, FieldSort, PrimaryID},
    },
    error::Error,
};

use super::source_type::SourceType;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Channel {
    pub id: i32,
    pub name: String,
    pub url: String,
    pub last_refresh: i64,
    pub last_successful_refresh: Option<i64>,
    pub refresh_frequency: i32,
    pub base_refresh_frequency: Option<i32>,
    pub source_type: SourceType,
    pub weight: f32,
}

impl PrimaryID<i32> for Channel {
    fn get_primary_id(&self) -> Option<i32> {
        Some(self.id)
    }
}

impl FieldSort<String> for Channel {
    fn sort_by_value(&self) -> String {
        self.name.clone()
    }
}

impl Channel {
    pub fn new(name: &str, url: &str, source: SourceType) -> Self {
        Channel {
            id: 0,
            name: name.to_string(),
            url: url.to_string(),
            last_refresh: 0,
            last_successful_refresh: Some(0),
            refresh_frequency: 60000,
            base_refresh_frequency: Some(60000),
            source_type: source,
            weight: 1.,
        }
    }
}

pub async fn new_with_seq_db(
    name: &str,
    url: &str,
    source: SourceType,
    channels_coll: &Channels<Channel>,
) -> Result<Channel, Error> {
    let mut channel = Channel::new(name, url, source);
    channel.id = channels_coll.get_next_seq().await?;
    channels_coll
        .insert_many(&[channel.clone()])
        .await
        .and(Ok(channel))
}
