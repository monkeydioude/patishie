use chrono::Utc;
use serde::{de::{self, Visitor}, Deserialize, Serialize};

use crate::{db::{channel::Channels, model::{CollectionModel, FieldSort, PrimaryID}}, error::Error};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum SourceType {
    RSSFeed,
    Bakery,
    Other,
}

impl Serialize for SourceType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
            serializer.serialize_str(match self {
                SourceType::RSSFeed => "rss_feed",
                SourceType::Bakery => "bakery",
                SourceType::Other => "other",
            })
    }
}

impl<'de> Deserialize<'de> for SourceType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
            struct SourceTypeVisitor;

        impl <'de> Visitor<'de> for SourceTypeVisitor {
            type Value = SourceType;
            
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("'rss_feed', 'bakery' or 'other'")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error, {
                match v {
                    "rss_feed" => Ok(SourceType::RSSFeed),
                    "bakery" => Ok(SourceType::Bakery),
                    "other" => Ok(SourceType::Other),
                    _ => Err(de::Error::unknown_variant(v, &["rss_feed", "bakery", "other"])),
                }
            }
        }
        deserializer.deserialize_str(SourceTypeVisitor)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Channel
{
    pub id: i32,
    pub name: String,
    pub last_refresh: i64,
    pub source_type: SourceType,
    pub refresh_frequency: i64,
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
    pub fn new(name: &str, source: SourceType) -> Self {
        Channel {
            id: 0,
            name: name.to_string(),
            last_refresh: Utc::now().timestamp_millis(),
            source_type: source,
            refresh_frequency: 2000,
            weight: 1.,
        }
    }
}

pub async fn new_with_seq_db(
    name: &str,
    source: SourceType,
    channels_coll: &Channels<'_, Channel>,
) -> Result<Channel, Error> {
    let mut channel = Channel::new(name, source);
    channel.id = channels_coll.get_next_seq().await?;
    channels_coll.insert_many(&[channel.clone()], Some("id".to_string()))
        .await
        .and(Ok(channel))
}