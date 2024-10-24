use super::{
    model::{CollectionModel, CollectionModelConstraint},
    mongo::Handle,
};
use crate::{
    entities::{
        channel::{new_with_seq_db, Channel},
        source_type::SourceType,
    },
    error::Error,
};
use chrono::{DateTime, Utc};
use mongodb::{bson::doc, results::InsertManyResult, Collection, Database, IndexModel};
use serde::Serialize;
use std::{fmt::Debug, sync::Arc};

#[derive(Debug)]
pub struct Channels<T: Serialize> {
    collection: Collection<T>,
    handle: Arc<Handle>,
    db_name: String,
}

impl Channels<Channel> {
    pub async fn insert_many(
        &self,
        data: &[Channel],
        index: Option<String>,
    ) -> Result<InsertManyResult, Error> {
        // let idx = index.unwrap_or_else(|| "create_date".to_string());
        // // Works cause we dont store result, nor do we return it.
        // // An Err() is returned, if that's the case.
        // self.collection()
        //     // Oftenly creating new collectionm therefore index
        //     .create_index(IndexModel::builder().keys(doc! {idx: -1}).build(), None)
        //     .await?;
        CollectionModel::<i32, Channel>::insert_many(self, data).await
    }

    pub fn get_database_name(&self) -> &String {
        &self.db_name
    }

    pub async fn update_refresh(
        &self,
        channel_id: impl Into<Option<i32>>,
        channel_name: impl Into<Option<&str>>,
        time: DateTime<Utc>,
        success: bool,
    ) -> Option<i64> {
        let mut doc = None;
        if let Some(id) = channel_id.into() {
            doc = Some(doc! {"id": id});
        } else if let Some(name) = channel_name.into() {
            doc = Some(doc! {"name": name});
        }
        let refresh_time = time.timestamp_millis();
        let update = match success {
            true => doc! {"$set": {
                "last_refresh": refresh_time,
                "last_successful_refresh": refresh_time,
            }},
            false => doc! {"$set": {
                "last_refresh": refresh_time,
            }},
        };
        let uw_doc = doc?;
        self.collection()
            .update_one(uw_doc, update, None)
            .await
            .ok()
            .and(Some(refresh_time))
    }

    pub async fn update_refresh_now(
        &self,
        channel_id: impl Into<Option<i32>>,
        channel_name: impl Into<Option<&str>>,
        success: bool,
    ) -> Option<i64> {
        self.update_refresh(channel_id, channel_name, Utc::now(), success)
            .await
    }

    pub fn new(handle: Arc<Handle>, db_name: &str) -> Result<Self, Error> {
        let collection = (match handle.database(db_name) {
            Some(res) => res,
            None => return Err(Error("no database found".to_string())),
        })
        .collection::<Channel>("channels");
        Ok(Channels {
            db_name: db_name.to_string(),
            handle,
            collection,
        })
    }
}

impl<P: PartialEq, T: CollectionModelConstraint<P>> CollectionModel<P, T> for Channels<T> {
    fn collection(&self) -> &Collection<T> {
        &self.collection
    }

    fn get_collection_name(&self) -> String {
        self.collection.name().to_string()
    }

    fn get_database(&self) -> Option<&Database> {
        self.handle.database(&self.db_name)
    }
}

pub async fn get_channel_id(
    channels_coll: &Channels<Channel>,
    channel_name: &str,
    channel_url: &str,
    source_type: SourceType,
) -> Result<i32, Error> {
    match channels_coll
        .find(doc! {"name": channel_name}, None, 1)
        .await
        .unwrap_or_default()
        .pop()
    {
        Some(p) => Ok(p.id),
        None => new_with_seq_db(channel_name, channel_url, source_type, channels_coll)
            .await
            .and_then(|el| Ok(el.id)),
    }
}
