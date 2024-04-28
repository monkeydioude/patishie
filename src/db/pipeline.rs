use mongodb::bson::{doc, to_bson, Document};
use serde::Serialize;

#[derive(Debug, Clone)]
pub enum FieldMatcher<'a, T: ?Sized + Serialize> {
    Lt(&'a T),
}

impl<'a, T: ?Sized + Serialize> FieldMatcher<'a, T> {
    pub fn setup(&self) -> Document {
        match self {
            FieldMatcher::Lt(data) => doc! {"$lt": to_bson(data).unwrap_or_default()},
        }
    }
}

#[derive(Debug, Clone)]
pub enum Pipe<'a, T: ?Sized + Serialize> {
    AddFields(&'a str, &'a[&'a str], FieldMatcher<'a, T>)
}

#[derive(Debug)]
pub struct Pipeline<'a, T: ?Sized + Serialize>(Vec<Pipe<'a, T>>);

impl<'a, T: ?Sized + Serialize + Clone> Pipeline<'a, T> {
    pub fn from_slice(pipes: &[Pipe<'a, T>]) -> Self {
       Pipeline(pipes.to_vec())
    }

    pub fn single_add_lt(
        target_field: &'a str,
        add_field: &'a [&str],
        value: &'a T,
    ) -> Vec<Document> {
        Pipeline::from_slice(&[
            Pipe::AddFields(target_field, add_field, FieldMatcher::Lt(value))
        ]).build()
    }

    pub fn new() -> Self {
        Pipeline(vec![])
    }

    pub fn push(&mut self, pipe: Pipe<'a, T>) -> &mut Self {
        self.0.push(pipe);
        self
    }

    pub fn build(&self) -> Vec<Document> {
        let mut add_ops = doc! {};
        let mut match_fields = doc!{};
        self.0
            .iter()
            .for_each(|pipe| {
                match pipe {
                    Pipe::AddFields(field_name, add_fields, match_field) => {
                        let field_string = field_name.to_string();
                        let af: Vec<String> = add_fields.to_vec().iter().map(|f| "$".to_string()+f).collect();
                        add_ops.insert(&field_string, doc! {"$add": af});
                        match_fields.insert(&field_string, match_field.setup())
                    },
                };
            });
        vec![doc! {"$addFields": add_ops}, doc! {"$match": match_fields}]
    }
} 

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use mongodb::bson::doc;

    use crate::db::pipeline::{FieldMatcher, Pipeline};

    use super::Pipe;

    #[test]
    fn test_pipelines_building() {
        let test_time = &Utc::now().timestamp_millis();
        assert_eq!(
            Pipeline::from_slice(&[Pipe::AddFields("field_a", &["a", "b"], FieldMatcher::Lt(&test_time))]).build(),
            vec! [doc! {"$AddFieldss": {"field_a": {"$add": ["$a", "$b"]}}}, doc! {"$match": {"field_a": {"$lt": &test_time}}}]
        );
        assert_eq!(
            Pipeline::single_add_lt("field_b", &["ac", "dc"], &test_time),
            vec! [doc! {"$AddFieldss": {"field_b": {"$add": ["$ac", "$dc"]}}}, doc! {"$match": {"field_b": {"$lt": &test_time}}}]
        );
    }
}