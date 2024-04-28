use chrono::{DateTime, Duration, Utc};
use url::{Url, Position};
use std::time::{SystemTime, UNIX_EPOCH};

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
