use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Content {
    pub url: String,
    pub description: Option<String>,
    pub credit: Option<String>
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Item {
    pub title: String,
    #[serde(alias = "pubDate")]
    pub pub_date: Option<String>,
    pub description: Option<String>,
    #[serde(alias = "creator")]
    pub creator: Option<String>,
    pub category: Option<Vec<String>>,
    pub link: Option<String>,
    #[serde(alias = "content")]
    pub content: Option<Content>,
}

impl Item {
    pub fn get_create_date(&self) -> i64 {
        if self.pub_date.is_none() {
            return 0;
        }
        self.pub_date
            .clone()
            .map(|pub_date| pub_date.parse::<DateTime<Utc>>().unwrap_or_default())
            .unwrap_or_default()
            .timestamp_millis()
    }
    pub fn get_img(&self) -> String {
        self.content
            .as_ref()
            .map(|c| c.url.clone())
            .unwrap_or_default()
    }
    pub fn get_desc(&self) -> String {
        self.description.clone().unwrap_or_default()
    }
    pub fn get_title(&self) -> Option<String> {
        Some(self.title.clone())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Channel {
    pub item: Vec<Item>,
    pub language: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(alias = "lastBuildDate")]
    pub last_build_date: Option<String>,
    #[serde(alias = "updatePeriod")]
    pub update_period: Option<String>,
    #[serde(alias = "updateFrequency")]
    pub update_frequency: Option<i32>,
}

impl Channel {
    pub fn get_channel_name(&self, url: &str) -> String {
       self.title.clone().unwrap_or_else( move || url.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Rss {
    pub channel: Channel,
}