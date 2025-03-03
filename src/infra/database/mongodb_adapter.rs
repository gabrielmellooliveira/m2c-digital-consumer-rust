use mongodb::{Client, Database, Collection};
use mongodb::options::ClientOptions;
use serde::Serialize;
use std::error::Error;

#[derive(Clone)]
pub struct MongoDbAdapter {
    client: Option<Client>,
    uri: String,
    database_name: String,
}

impl MongoDbAdapter {
    pub fn new(uri: String, database_name: String) -> Self {
        MongoDbAdapter {
            client: None,
            uri,
            database_name
        }
    }

    pub async fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        let client_options = ClientOptions::parse(&self.uri).await?;
        
        let client = Client::with_options(client_options)?;
        self.client = Some(client);
        
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(client) = &self.client {
            self.client = None;
        }
        Ok(())
    }

    pub fn get_collection<T>(&self, name: &str) -> Option<Collection<T>> {
        self.client.as_ref().map(|client| {
            let db = client.database("m2c_digital_db"); // Substitua pelo nome do seu banco de dados
            db.collection(name)
        })
    }

    pub async fn insert_one<T>(&self, collection_name: &str, document: &T) -> Result<(), Box<dyn Error>>
    where
        T: Serialize,
    {
        if let Some(collection) = self.get_collection::<T>(collection_name) {
            collection.insert_one(document, None).await?;
            Ok(())
        } else {
            Err("Failed to connect to MongoDB".into())
        }
    }
}