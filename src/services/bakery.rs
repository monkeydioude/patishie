use chrono::Utc;
use reqwest::Client;
use uuid::Uuid;

use crate::converters::string::to_articles;
use crate::entities::potential_articles::PotentialArticle;

const X_REQUEST_ID_LABEL: &str = "X-Request-ID";
const NO_X_REQUEST_ID_LABEL: &str = "no_x_request_id";

/// get_cookies_from_bakery calls bakery, a website scrapper, and returns the result
pub async fn get_cookies_from_bakery(
    api_path: &str,
    channel_url: &str,
    uuid: Uuid,
) -> Option<Vec<PotentialArticle>> {
    let client = Client::new();
    let mut uuid_str = uuid.to_string();
    if uuid_str == "" {
        uuid_str = NO_X_REQUEST_ID_LABEL.to_string();
    }
    // let response = reqwest::get(format!("{}/bakery?url={}", api_path, url)).await;
    let response = client
        .get(format!("{}/bakery?url={}", api_path, channel_url))
        .header(X_REQUEST_ID_LABEL, uuid_str)
        .send()
        .await;
    let raw_data = match response {
        Ok(res) => res.text().await.unwrap_or("[]".to_string()),
        Err(err) => {
            eprintln!("[{}] ({}) {}", uuid, Utc::now(), err);
            return None;
        }
    };
    Some(to_articles(&raw_data))
}
