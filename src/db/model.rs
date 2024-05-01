use crate::error::Error;
use futures::{StreamExt, TryStreamExt};
use mongodb::{
    bson::{doc, Document},
    options::{FindOneAndUpdateOptions, FindOptions},
    results::InsertManyResult,
    Collection, Database,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{collections::HashMap, fmt::{Debug, Display}, sync::Arc, vec};

use super::mongo::{db_not_found_err, to_bson_vec, Handle};

#[derive(Copy, Clone, Debug)]
pub enum SortOrder {
    ASC = 1,
    DESC = -1,
}

impl SortOrder {
    pub fn value(&self) -> i32 {
        *self as i32
    }
}

impl From<Option<SortOrder>> for SortOrder {
    fn from(value: Option<SortOrder>) -> Self {
        value.unwrap_or(SortOrder::ASC)
    }
}

pub trait CollectionModelConstraint<P: PartialEq>:
    Serialize + FieldSort<String> + PrimaryID<P> + Debug + Unpin + Send + Sync + DeserializeOwned + Clone
{
}
impl<P: PartialEq, T> CollectionModelConstraint<P> for T where
    T: Serialize + FieldSort<String> + PrimaryID<P> + Debug + Unpin + Send + Sync + DeserializeOwned + Clone
{
}

#[derive(Debug, Serialize, Deserialize)]
struct Counter {
    _id: String,
    seq: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct DocsWrapper<T> {
    docs: Vec<T>,
}

pub trait CollectionModel<P: PartialEq, T: CollectionModelConstraint<P>> {
    async fn find_aggregate(&self, pipeline: Vec<Document>) -> Option<Vec<T>> {
        let mut cursor = self.collection()
            .aggregate(pipeline, None)
            .await
            .map_err(|err| {
                eprintln!( "model::CollectionModel::find_aggregate could not find latest: {}", err);
                err
            })
            .ok()?;
        let mut results = Vec::<T>::new();
        while let Some(doc) = cursor.next().await {
            match doc {
                Ok(doc) => match mongodb::bson::from_document::<T>(doc) {
                    Ok(t) => results.push(t.clone()),
                    Err(e) => eprintln!("model::CollectionModel::find_aggregate failed to deserialize document: {}", e),
                },
                Err(e) => eprintln!("model::CollectionModel::find_aggregate failed to retrieve document: {}", e),
            }
        }
        Some(results)
    }

    // async fn update_one(&self, updates: &[(&str, )])

    /// insert_many inserts an array of documents into the collection
    async fn insert_many(&self, data: &[T]) -> Result<InsertManyResult, Error> {
        if data.is_empty() {
            return Error::to_result_string("empty input")?;
        }

        self.collection()
            .insert_many(data, None)
            .await
            .map_err(Error::from)
    }

    /// find_by_field_values fetch a `limit` number of documents matching a `field`
    async fn find_by_field_values(&self, data: &[T], field: &str, limit: i64) -> Vec<T> {
        let mut in_values = vec![];
        for item in data {
            in_values.push(item.sort_by_value());
        }

        let filter = doc! {field: { "$in": in_values }};
        let mut cursor = match self
            .collection()
            .find(
                filter,
                FindOptions::builder()
                    .limit(limit)
                    .sort(doc! {"_id": SortOrder::DESC.value()})
                    .build(),
            )
            .await
        {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let mut results = vec![];
        while let Some(Ok(res)) = cursor.next().await {
            results.push(res);
        }

        results
    }

    /// find_all is a short for self.find(None, None, None)
    /// which retrieves every document from a collection
    async fn find_all(&self) -> Option<Vec<T>> {
        self.find(None, None, None).await
    }

    /// find returns document matching a `doc`, sorting on a `field` using a `sort` order (SortOrder),
    /// limited to a `limit` number of documents.
    async fn find(
        &self,
        filter: impl Into<Option<Document>>,
        sort: impl Into<Option<(&str, SortOrder)>>,
        limit: impl Into<Option<i64>>,
    ) -> Option<Vec<T>> {
        let sort_values = sort.into().unwrap_or_else(|| ("_id", SortOrder::DESC));
        let find_options = FindOptions::builder()
            .limit(limit)
            .sort(doc! {
                sort_values.0: sort_values.1.value(),
            })
            .build();

        match self
            .collection()
            .find(filter.into().unwrap_or_else(|| doc!{}), find_options)
            .await
            .map_err(|err| {
                eprintln!(
                    "model::CollectionModel::find_latests could not find latest: {}",
                    err
                );
                err
            })
            .ok()
        {
            Some(mut cursor) => {
                let mut results = vec![];
                while let Some(Ok(res)) = cursor.next().await {
                    results.push(res);
                }
                Some(results)
            }
            None => None,
        }
    }
    /// find_with_limits allows to use multiple fields to request documents through the `field_in` parameters.
    /// It also allows to define different limit for the different fields used through the `limits_in` parameter.
    /// For example, I want 10 documents matching the field "channel_id": 1,
    /// and 30 documents matching the field "channel_id": 2, ordered DESC by `created_at`:
    ///
    /// find_with_limits("a_field", vec![1, 2], HashMap::from([(1, 10), (2, 30)]), 10, Some("created_at", SortOrder::DESC))
    async fn find_with_limits<L: Eq + PartialEq<P> + Ord + PartialOrd + Sized + Debug + Display>(
        &self,
        field: &str,
        field_in: Vec<i32>,
        limits_in: impl Into<Option<HashMap<L, i64>>>,
        mut max_limit: i64,
        sort_tuple: impl Into<Option<(&str, SortOrder)>>,
    ) -> Option<Vec<T>> {
        let mut limits_safe = HashMap::new();
        if let Some(limits_in_into) = limits_in.into() {
            limits_safe = limits_in_into;
            max_limit = limits_safe
            .iter()
            .map(|e| *e.1)
            .max()
            .unwrap_or(max_limit);
        }
        let mut pipeline = vec![
            doc! { "$match": { field: { "$in": to_bson_vec(&field_in) } } },
            doc! { "$group": {
                "_id": format!("${}", field),
                "docs": { "$push": "$$ROOT" }
            }},
            doc! { "$project": {
                "_id": 0,
                "link": 1,
                "docs": { "$slice": ["$docs", max_limit * field_in.len() as i64] }
            }}
        ];
        if let Some((field_name, order)) = sort_tuple.into() {
            pipeline.insert(1, doc! { "$sort": {field_name: order.value()} });
        }
        let mut cursor = self.collection()
            .aggregate(pipeline, None)
            .await
            .map_err(|err| {
                eprintln!( "model::CollectionModel::find_with_limits could not find latest: {}", err);
                err
            })
            .ok()?;
        let mut results = Vec::<T>::new();
        while let Some(doc) = cursor.next().await {
            match doc {
                Ok(doc) => match mongodb::bson::from_document::<DocsWrapper<T>>(doc) {
                    Ok(t) => {
                        let limit = t.docs.last()
                            .and_then(|t_item| t_item.get_primary_id())
                            .and_then(|p_item| limits_safe.iter().find(|el| el.0 == &p_item).map(|l| *l.1))
                            .unwrap_or(max_limit);
                        t.docs.iter().take(limit as usize).for_each(|inner_doc| results.push(inner_doc.clone()))
                    },
                    Err(e) => eprintln!("model::CollectionModel::find_with_limits failed to deserialize document: {}", e),
                },
                Err(e) => eprintln!("model::CollectionModel::find_with_limits failed to retrieve document: {}", e),
            }
        }
        Some(results)
    }
    /// find_latests returns a `limit` amount of documents
    /// ordered by `sort` (SortOrder) on a `field`.
    /// If `field` is an integer, `after` can be used to fetch
    /// documents that are greater than `field`.
    async fn find_latests(
        &self,
        field: &str,
        after: impl Into<Option<i64>>,
        limit: impl Into<Option<i64>>,
        sort: impl Into<Option<SortOrder>>,
        filter: impl Into<Option<Document>>,
    ) -> Option<Vec<T>> {
        if field.is_empty() {
            return None;
        }
        let find_options = FindOptions::builder()
            .limit(limit)
            .sort(doc! {
                field: sort.into().unwrap_or(SortOrder::DESC).value(),
            })
            .build();
        let mut filter_options = match filter.into() {
            Some(d) => d,
            None => doc!{},
        };
        let after_into = after.into();
        if after_into.is_some() {
            filter_options.insert(field, doc! {
                "$gt": after_into.unwrap(),
            });
        }

        self
            .collection()
            .find(filter_options, find_options)
            .await
            .map_err(|err| {
                eprintln!( "model::CollectionModel::find_latests could not find latest: {}", err);
                err
            })
            .ok()?
            .try_collect()
            .await
            .map_err(|err| {
                eprintln!( "model::CollectionModel::find_latests could collect: {}", err);
                err
            })
            .ok()
    }

    fn collection(&self) -> &Collection<T>;
    fn get_collection_name(&self) -> String;
    fn get_database(&self) -> Option<&Database>;
    fn get_diff_collection<C>(&self, coll: &str) -> Option<Collection<C>> {
        Some(self.get_database()?.collection::<C>(coll))
    }
    /// get_next_seq requires a "counters" collection to exist, or writing rights to create it.
    /// It will then try to fetch a document, containing a `seq` field, matching the collection's name as its `_id`.
    /// If does not exist, a new document with the previously mentioned specifics will be created, setting `seq` to `0`.
    /// Then, `seq` will be incremented by 1, and the document updated in the collection.
    /// Finally, the updated `seq` will be returned.
    async fn get_next_seq(&self) -> mongodb::error::Result<i32> {
        let counters = self.get_diff_collection::<Counter>("counters")
            .ok_or(db_not_found_err())?;
        let filter = doc! { "_id": self.get_collection_name() };
        let update = doc! {
            "$inc": { "seq": 1 }, 
            "$setOnInsert": { "_id": self.get_collection_name() }
        };
        let options = FindOneAndUpdateOptions::builder()
            .upsert(true)
            .return_document(mongodb::options::ReturnDocument::After)
            .build();

        let result_doc = counters
            .find_one_and_update(filter, update, options)
            .await?
            .ok_or_else(|| db_not_found_err());

        result_doc.map(|doc| doc.seq)
    }

    async fn get_seq(&self, id: &str) -> mongodb::error::Result<i32> {
        let counters = self.get_diff_collection::<Counter>("counters")
            .ok_or(db_not_found_err())?;
        let res = counters
            .find_one(doc! {"_id": id}, None)
            .await?;
        match res {
            Some(r) => Ok(r.seq),
            None => self.get_next_seq().await,
        }
    }
}

pub trait FieldSort<V> {
    fn sort_by_value(&self) -> V;
}

pub trait PrimaryID<T: PartialEq> {
    fn get_primary_id(&self) -> Option<T>;
}

pub struct BlankCollection<T: Serialize> {
    collection: Collection<T>,
    handle: Arc<Handle>,
    db_name: String,
}

impl<P: PartialEq, T: CollectionModelConstraint<P>> CollectionModel<P, T> for BlankCollection<T> {
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

impl<T: CollectionModelConstraint<String>> BlankCollection<T> {
    pub fn new(handle: Arc<Handle>, db_name: &str, collection_name: &str) -> Result<Self, Error> {
        let collection = (match handle.database(db_name) {
            Some(res) => res,
            None => return Error::to_result_string("no database found"),
        })
        .collection::<T>(collection_name);

        Ok(Self {
            db_name: db_name.to_string(),
            handle,
            collection,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use serde::{Deserialize, Serialize};
    use crate::{config, db::{self, items::Items, model::{CollectionModel, SortOrder}}, entities::potential_articles::PotentialArticle};
    use super::{BlankCollection, FieldSort, PrimaryID};

    #[derive(Clone, Debug, Serialize, Deserialize)]
    struct Test {
        link: String,
    }
    impl FieldSort<String> for Test {
        fn sort_by_value(&self) -> String {
            "pog".to_string()
        }
    }

    impl PrimaryID<String> for Test {
        fn get_primary_id(&self) -> Option<String> {
            None
        }
    }
    
    #[tokio::test]
    async fn test_get_seq() {
        let settings = config::Settings::new().unwrap();
        let db_handle = Arc::new(db::mongo::get_handle(&settings).await);
        let coll = BlankCollection::<Test>::new(db_handle, "panya", "test").unwrap();
        // coll.insert_many(&[Test{}]).await.unwrap();

        println!("next sequence: {}", coll.get_next_seq().await.unwrap() );
    }

    #[tokio::test]
    async fn test_find_with_limits() {
        let settings = config::Settings::new().unwrap();
        let db_handle = Arc::new(db::mongo::get_handle(&settings).await);
        let coll = Items::<PotentialArticle>::new(db_handle, "panya").unwrap();
        // coll.insert_many(&[Test{}]).await.unwrap();

        println!("next find_with_limits: {:?}", coll.find_with_limits(
			"channel_id",
			vec![1, 2], 
			HashMap::from([
                (1, 10),
                (2, 5),
            ]),
			10,
			("create_date", SortOrder::DESC),
        ).await.unwrap());
    }
}
