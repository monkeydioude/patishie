use std::collections::HashMap;
use mongodb::{bson::Bson, options::ClientOptions, Client, Database};
use crate::config::Settings;

#[derive(Debug, Clone)]
pub struct Handle {
    client: Client,
    databases: HashMap<String, Database>,
}

impl Handle {
    pub fn client(&self) -> Option<&Client> {
        Some(&self.client)
    }

    pub fn database(&self, db_name: &str) -> Option<&Database> {
        self.databases.get(db_name)
    }


    pub async fn new(settings: &Settings) -> Self {
        let mut client_options = ClientOptions::parse(&settings.db_path).await.unwrap();
        client_options.app_name = Some(settings.app_name.clone());
        let client = Client::with_options(client_options).unwrap();
        let databases = settings
            .databases
            .iter()
            .map(|name| (name.clone(), client.database(name)))
            .collect();

        Handle { client, databases }
    }
    
}

pub fn to_bson_vec(vec: &Vec<i32>) -> Vec<Bson> {
    vec.iter().map(|&id| Bson::from(id)).collect::<Vec<Bson>>()
}

pub async fn get_handle(settings: &Settings) -> Handle {
    Handle::new(settings).await
}

pub fn db_not_found_err() -> mongodb::error::Error {
    mongodb::error::Error::from(std::io::ErrorKind::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_bson_vec() {
        assert_eq!(to_bson_vec(&vec![1, 2]), vec![Bson::Int32(1), Bson::Int32(2)]);
    }
}
