use redis::{AsyncCommands, Client, aio::Connection};
use tokio::sync::Mutex;
use std::{error::Error, sync::Arc};

#[derive(Clone)]
pub struct RedisAdapter {
    client: Client,
    connection: Option<Arc<Mutex<Connection>>>,
}

impl RedisAdapter {
    pub fn new(url: String) -> Result<Self, Box<dyn Error>> {
        let client = Client::open(url)?;
        Ok(Self {
            client,
            connection: None,
        })
    }

    pub async fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        let conn = self.client.get_async_connection().await?;
        self.connection = Some(Arc::new(Mutex::new(conn)));
        Ok(())
    }

    pub async fn disconnect(&mut self) {
        self.connection = None;
    }

    pub async fn set(&mut self, key: &str, value: &str) -> Result<(), Box<dyn Error>> {
        match &mut self.connection {
            Some(conn) => {
                conn.lock().await.set::<&str, &str, ()>(key, value).await?;
                Ok(())
            }
            None => Err("Failed to set value in Redis".into()),
        }
    }

    pub async fn get(&mut self, key: &str) -> Result<String, Box<dyn Error>> {
        match &mut self.connection {
            Some(conn) => {
                let value: String = conn.lock().await.get::<&str, String>(key).await?;
                Ok(value)
            }
            None => Err("Failed to get value from Redis".into()),
        }
    }

    pub async fn increment(&mut self, key: &str) -> Result<(), Box<dyn Error>> {
        match &mut self.connection {
            Some(conn) => {
                conn.lock().await.incr::<&str, i64, ()>(key, 1).await?;
                Ok(())
            }
            None => Err("Failed to increment key in Redis".into()),
        }
    }

    pub async fn delete(&mut self, key: &str) -> Result<(), Box<dyn Error>> {
        match &mut self.connection {
            Some(conn) => {
                conn.lock().await.del::<&str, String>(key).await?;
                Ok(())
            }
            None => Err("Failed to delete key in Redis".into()),
        }
    }
}