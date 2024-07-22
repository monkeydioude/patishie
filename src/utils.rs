use chrono::{DateTime, Duration, Utc};
use url::{Url, Position};
use std::{fmt::Display, ops::Add, sync::Arc, time::{SystemTime, UNIX_EPOCH}};

use crate::{db::{channel::Channels, entities::Timer, items::Items, model::BlankCollection, mongo::Handle, timers::Timers}, entities::{channel::Channel, potential_articles::PotentialArticle}, error::Error};

pub struct DBBag {
    // db_handle: Arc<Handle>,
    pub channels_coll: Channels<Channel>,
    pub items_coll: Items<PotentialArticle>,
    pub timers_coll: BlankCollection<Timer>,
} 

impl DBBag {
    pub fn new(db_handle: Arc<Handle>) -> Result<Self, Error> {
        Ok(Self {
            // db_handle: db_handle.clone(),
            channels_coll: Channels::<Channel>::new(db_handle.clone(), "panya")?,
            items_coll: Items::<PotentialArticle>::new(db_handle.clone(), "panya")?,
            timers_coll: Timers::new(db_handle, "panya", "timers")?,
        })
    }
}

pub fn now_timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

pub fn now_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}


pub fn datetime_minus_minutes(minus_minutes: i64, dt: DateTime<Utc>) -> i64 {
    (dt - Duration::minutes(minus_minutes)).timestamp()
}

pub fn now_minus_minutes(minus_minutes: i64) -> i64 {
    datetime_minus_minutes(minus_minutes, Utc::now())
}

pub fn clean_url(input_url: &str) -> Result<String, url::ParseError> {
    let mut url = Url::parse(input_url)?;
    url.set_query(None);
    url.set_fragment(None);
    let _ = url.set_scheme("");
    let mut cleaned_url = url[..Position::AfterPath].to_string();

    if cleaned_url.ends_with('/') {
        cleaned_url.pop();
    }
    let scheme_end = cleaned_url.find("://").map(|i| i + 3).unwrap_or(0);

    let cleaned_url_no_scheme = &cleaned_url[scheme_end..];

    Ok(cleaned_url_no_scheme.to_string())
}

pub trait MeasureUnit {
    fn unit(&self) -> String;
}

#[derive(Clone, Copy)]
pub struct Millisecond(pub u64);

impl Millisecond {
    pub fn sec(&self) -> Second {
        Second(self.0 / 1000)
    }
}

impl Add<u64> for Millisecond {
    type Output = u64;

    fn add(self, rhs: u64) -> Self::Output {
        self.clone().0 + rhs
    }
}

impl MeasureUnit for Millisecond {
    fn unit(&self) -> String {
        "ms".to_string()
    }
}

impl Display for Millisecond {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.0, self.unit())
    }
}

impl From<u64> for Millisecond {
    fn from(value: u64) -> Self {
        Millisecond(value)
    }
}

impl From<Millisecond> for u64 {
    fn from(value: Millisecond) -> Self {
        value.0
    }
}

#[derive(Clone, Copy)]
pub struct Second(pub u64);

impl Second {
    pub fn msec(&self) -> Millisecond {
        Millisecond(self.0 * 1000)
    }
}

impl From<u64> for Second {
    fn from(value: u64) -> Self {
        Second(value)
    }
}

impl From<Second> for u64 {
    fn from(value: Second) -> Self {
        value.0
    }
}

impl Add<u64> for Second {
    type Output = u64;

    fn add(self, rhs: u64) -> Self::Output {
        self.clone().0 + rhs
    }
}

impl Display for Second {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.0, self.unit())
    }
}

impl MeasureUnit for Second {
    fn unit(&self) -> String {
        "s".to_string()
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn test_datetime_minus_x_minutes() {
        let tt = 1696769957;
        let mins = 2;

        assert_eq!(
            datetime_minus_minutes(mins, Utc.timestamp_opt(tt, 0).unwrap()),
            tt - (60 * mins),
        );
    }

    #[test]
    fn test_i_can_clean_url() {
        let trial = "https://www3.nhk.or.jp/news/easy/?limit=5";
        let goal = "www3.nhk.or.jp/news/easy";

        assert_eq!(clean_url(trial).unwrap(), goal);
    }
}
