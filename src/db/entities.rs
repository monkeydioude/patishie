use super::model::{FieldSort, PrimaryID};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Timer {
    pub channel: String,
    pub update_date: i64,
    pub time_to_refresh: i64,
}

impl FieldSort<String> for Timer {
    fn sort_by_value(&self) -> String {
        self.channel.clone()
    }
}

impl PrimaryID<String> for Timer {
    fn get_primary_id(&self) -> Option<String> {
        Some(self.channel.clone())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Refresh {
    pub url: String,
    pub last_refresh_date: i64,
    // ms
    pub refresh_frequency: i64,
    pub update_date: i64,
    pub fetch_avg: f32,
    pub limit: i32,
}

impl FieldSort<String> for Refresh {
    fn sort_by_value(&self) -> String {
        self.last_refresh_date.to_string()
    }
}

impl PrimaryID<String> for Refresh {
    fn get_primary_id(&self) -> Option<String> {
        Some(self.url.clone())
    }
}


#[derive(Deserialize)]
pub enum AscDesc {
    ASC,
    DESC,
}

impl AscDesc {
    pub fn as_str(&self) -> &'static str {
        match self {
            AscDesc::ASC => "ASC",
            AscDesc::DESC => "DESC"
        }
    }
}