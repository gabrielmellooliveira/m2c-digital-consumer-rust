mod infra {
    pub mod queue {
        pub mod rabbitmq_adapter;
    }

    pub mod http {
        pub mod http_adapter;
    }

    pub mod database {
        pub mod mongodb_adapter;
        pub mod redis_adapter;
    }
}

mod usecases {
    pub mod consume_messages;
}

mod domain {
    pub mod models {
        pub mod message;
    }
}

use infra::queue::rabbitmq_adapter::RabbitMqAdapter;
use infra::database::mongodb_adapter::MongoDbAdapter;
use infra::database::redis_adapter::RedisAdapter;
use infra::http::http_adapter::HttpAdapter;
use usecases::consume_messages::ConsumeMessagesUseCase;

use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let m2c_digital_api_url = env::var("M2C_DIGITAL_API_URL").expect("M2C_DIGITAL_API_URL not found");
    let m2c_digital_api_key = env::var("M2C_DIGITAL_API_KEY").expect("M2C_DIGITAL_API_KEY not found");
    let mongodb_url = env::var("MONGODB_URL").expect("MONGODB_URL not found");
    let rabbitmq_url = env::var("RABBITMQ_URL").expect("RABBITMQ_URL not found");
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL not found");

    let rabbitmq_adapter = RabbitMqAdapter::new(rabbitmq_url);
    let mut mongodb_adapter = MongoDbAdapter::new(mongodb_url, "m2c_digital_db".to_string());
    let mut redis_adapter = RedisAdapter::new(redis_url).expect("");
    let mut http_adapter = HttpAdapter::new(m2c_digital_api_url);

    http_adapter.add_header("x-api-key", m2c_digital_api_key.as_str());

    if let Err(e) = mongodb_adapter.connect().await {
        eprintln!("Failed to connect to MongoDB: {}", e);
        return;
    }

    if let Err(e) = redis_adapter.connect().await {
        eprintln!("Failed to connect to Redis: {}", e);
        return;
    }

    let consume_messages = ConsumeMessagesUseCase::new(
        mongodb_adapter,
        http_adapter,
        rabbitmq_adapter,
        redis_adapter,
    );

    consume_messages.execute().await.expect("Failed to consume messages");
}