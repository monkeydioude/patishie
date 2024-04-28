use crate::{
    db::{
        channel::{get_channel_id, Channels}, items::Items, model::{CollectionModel, SortOrder}
    }, entities::{channel::{Channel, SourceType}, potential_articles::PotentialArticle}, services::vec::RemoveReplaceExisting
};
use mongodb::bson::doc;
/// process_data_and_fetch_items compares fetched articles from bakery against existing ones in DB,
/// then insert those not existing and then returns the latest `limit` number of articles.
pub async fn process_data_and_fetch_items(
    articles: &Vec<PotentialArticle>,
    items_coll: Items<'_, PotentialArticle>,
    channels_coll: &'_ Channels<'_, Channel>,
    channel_name: &str,
    limit: i64,
) -> Vec<PotentialArticle> {
    // find existing links
    let existing_links = items_coll.find_by_field_values(&articles, "link", 0).await;
    // picks out existing links in db
    let mut to_insert = articles.remove_existing(&existing_links);


    // something to insert
    if !to_insert.is_empty() {
        let channel_id = match get_channel_id(&channels_coll, channel_name, SourceType::Bakery).await {
            Ok(r) => r,
            Err(_) => {
                eprintln!("could not find any channel_id for: {}", channel_name);
                return vec![];
            },
        };
        to_insert
            .iter_mut()
            .for_each(|pa| {
                pa.channel_name = Some(channel_name.to_string());
                pa.channel_id = Some(channel_id);
            });
        let _ = items_coll.insert_many(&to_insert, None).await;
    }
    items_coll
        .find_latests(
            "create_date", 
            None, 
            limit, 
            SortOrder::DESC,
            doc! {"channel_name": channel_name}
        )
        .await
        .unwrap_or(vec![])
}
