use std::fmt::Display;

use serde::{de::{self, Visitor}, Deserialize, Serialize};

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
            serializer.serialize_str(&format!("{}", self))
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

impl Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            SourceType::RSSFeed => "rss_feed",
            SourceType::Bakery => "bakery",
            SourceType::Other => "other",
        })
    }
}