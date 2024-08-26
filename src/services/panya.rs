use crate::{
    db::{
        channel::{get_channel_id, Channels},
        items::Items,
        model::CollectionModel,
    },
    entities::{channel::Channel, potential_articles::PotentialArticle, source_type::SourceType},
    error::Error,
    services::vec::RemoveReplaceExisting,
};
/// process_data_and_fetch_items compares fetched articles from bakery against existing ones in DB,
/// then insert those not existing and then returns the latest `limit` number of articles.
pub async fn process_data(
    articles: &Vec<PotentialArticle>,
    items_coll: &Items<PotentialArticle>,
    channels_coll: &Channels<Channel>,
    channel_name: &str,
    channel_url: &str,
    source_type: SourceType,
) -> Result<(), Error> {
    // find existing links
    let existing_links = items_coll.find_by_field_values(&articles, "link", 0).await;
    // picks out existing links in db
    let mut to_insert = articles.remove_existing(&existing_links);

    // something to insert
    if !to_insert.is_empty() {
        let channel_id =
            get_channel_id(&channels_coll, channel_name, channel_url, source_type).await?;
        to_insert.iter_mut().for_each(|pa| {
            pa.channel_name = Some(channel_name.to_string());
            pa.channel_id = Some(channel_id);
        });
        return items_coll.insert_many(&to_insert, None).await.map(|res| {
            println!("items_coll.insert_many{:?}", res);
            return ();
        });
    }
    Ok(())
}
