use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::Value;
use tokio::task::JoinHandle;
use crate::infra::database::mongodb_adapter::MongoDbAdapter;
use crate::infra::queue::rabbitmq_adapter::RabbitMqAdapter;
use crate::infra::http::http_adapter::HttpAdapter;
use crate::infra::database::redis_adapter::RedisAdapter;
use crate::domain::models::message::Message;

#[derive(Clone)]
pub struct ConsumeMessagesUseCase {
    mongo_db_adapter: MongoDbAdapter,
    http_adapter: HttpAdapter,
    rabbitmq_adapter: RabbitMqAdapter,
    redis_adapter: RedisAdapter,
    collection_name: String,
    queue_name: String,
}

impl ConsumeMessagesUseCase {
    pub fn new(
        mongo_db_adapter: MongoDbAdapter,
        http_adapter: HttpAdapter,
        rabbitmq_adapter: RabbitMqAdapter,
        redis_adapter: RedisAdapter,
    ) -> Self {
        Self {
            mongo_db_adapter,
            http_adapter,
            rabbitmq_adapter,
            redis_adapter,
            collection_name: "messages".to_string(),
            queue_name: "m2c_digital_messages_queue".to_string(),
        }
    }

    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        let queue_name = self.queue_name.clone();
        
        let processor = Arc::new(Mutex::new(self.clone()));

        let consume_callback = move |message: String| -> JoinHandle<()> {
            let processor = Arc::clone(&processor);

            tokio::spawn(async move {
                if let Ok(json_message) = serde_json::from_str::<Value>(&message) {
                    println!("Message: {}", json_message);
                    if let Some(data) = json_message.get("data") {
                        let mut processor = processor.lock().await;
                        processor.process_message(data.clone()).await.ok();
                    }
                }
            })
        };

        self.rabbitmq_adapter.consume(&queue_name, consume_callback).await?;
        Ok(())
    }

    async fn process_message(&mut self, message: Value) -> Result<(), Box<dyn std::error::Error>> {
        self.save_message(message.clone()).await?;

        let campaign_id = message["campaignId"].as_str().unwrap_or("").to_string();
        let total = message["total"].as_i64().unwrap_or(0) as i32;
        let campaign_key = format!("campaign:{}:count", campaign_id);

        if self.have_all_messages_been_read(&campaign_key, total).await? {
            self.update_campaign_status_to_sent(&campaign_id).await?;
            self.redis_adapter.delete(&campaign_key).await?;
        }

        Ok(())
    }

    async fn save_message(&mut self, message_data: Value) -> Result<(), Box<dyn std::error::Error>> {
        let message = Message::build(message_data).expect("");
        self.mongo_db_adapter.insert_one(&self.collection_name, &message).await?;
        Ok(())
    }

    async fn have_all_messages_been_read(&mut self, campaign_key: &str, total: i32) -> Result<bool, Box<dyn std::error::Error>> {
        self.redis_adapter.increment(campaign_key).await?;
        let message_count = self.redis_adapter.get(campaign_key).await?;
        Ok(total == message_count.parse::<i32>()?)
    }

    async fn update_campaign_status_to_sent(&mut self, campaign_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("/campaigns/sent/{}", campaign_id);
        self.http_adapter.put(&url, None).await?;
        Ok(())
    }
}
