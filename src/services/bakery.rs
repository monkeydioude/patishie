use crate::converters::string::to_articles;
use crate::entities::potential_articles::PotentialArticle;

/// get_cookies_from_bakery calls bakery, a website scrapper, and returns the result
pub async fn get_cookies_from_bakery(api_path: &str, url: &str) -> Option<Vec<PotentialArticle>> {
    let response = reqwest::get(format!("{}/bakery?url={}", api_path, url)).await;
    let raw_data = match response {
        Ok(res) => res.text().await.unwrap_or("[]".to_string()),
        Err(err) => {
            eprintln!("{}", err);
            return None;
        }
    };
    Some(to_articles(&raw_data))
}
