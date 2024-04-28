use crate::entities::potential_articles::PotentialArticle;

pub fn to_articles(raw_data: &str) -> Vec<PotentialArticle>
{
    serde_json::from_str::<Vec<PotentialArticle>>(raw_data).unwrap_or_else(|err| {
        eprintln!("could not deserialize into articles: {}", err);
        vec![]
    })
}