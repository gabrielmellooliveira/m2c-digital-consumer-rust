use lapin::{Connection, ConnectionProperties, options::{QueueDeclareOptions, BasicConsumeOptions}, types::FieldTable};
use futures_util::stream::StreamExt;

#[derive(Clone)]
pub struct RabbitMqAdapter {
    connection_url: String,
}

impl RabbitMqAdapter {
    pub fn new(connection_url: String) -> Self {
        RabbitMqAdapter { connection_url }
    }

    pub async fn connect(&self) -> Result<Connection, lapin::Error> {
        let connection = Connection::connect(&self.connection_url, ConnectionProperties::default()).await?;
        Ok(connection)
    }

    pub async fn close(connection: &Connection) -> Result<(), lapin::Error> {
        connection.close(0, "Bye").await?;
        Ok(())
    }

    pub async fn consume<F>(&self, domain_name: &str, callback: F) -> Result<(), lapin::Error>
    where
        F: Fn(String) -> tokio::task::JoinHandle<()>,
    {
        let connection = self.connect().await?;
        let channel = connection.create_channel().await?;

        channel.queue_declare(domain_name, QueueDeclareOptions {
            durable: true,
            ..Default::default()
        }, FieldTable::default()).await?;

        let mut consumer = channel.basic_consume(domain_name, "consumer", BasicConsumeOptions::default(), FieldTable::default()).await?;

        while let Some(message) = consumer.next().await {
            match message {
                Ok((channel, delivery)) => {
                    let body = String::from_utf8_lossy(&delivery.data);
                    let callback_handle = callback(body.to_string());
                    channel.basic_ack(delivery.delivery_tag, lapin::options::BasicAckOptions::default()).await?;
                    callback_handle.await;
                }
                Err(err) => {
                    eprintln!("Failed while consuming message: {:?}", err);
                }
            }
        }

        Ok(())
    }
}