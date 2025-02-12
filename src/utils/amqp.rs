use lapin::{BasicProperties, Connection, ConnectionProperties, Channel, options::BasicPublishOptions};
use super::base::AMQPConfig;
use super::client::EMLData;
use serde_json;


pub struct AMQPClient {
    channel: Channel,
    exchange: String,
    routing_key: String,
}


impl AMQPClient {
    pub async fn new(amqp_details: &AMQPConfig) -> Self {
        let amqp_url = format!(
            "amqp://{}:{}@{}:{}/{}",
            amqp_details.username,
            amqp_details.password,
            amqp_details.host,
            amqp_details.port,
            amqp_details.vhost
        );

        let connection = Connection::connect(&amqp_url, ConnectionProperties::default())
            .await
            .expect("Failed to connect to RabbitMQ");

        let channel = connection.create_channel()
            .await
            .expect("Failed to create channel");

        Self { channel, exchange: amqp_details.exchange.clone(), routing_key: amqp_details.routing_key.clone() }
    }

    pub async fn publish(&self, eml_data: &EMLData) {
        let payload = serde_json::to_vec(&eml_data).expect("Failed to serialize EMLData");

        self.channel
            .basic_publish(
                &self.exchange,
                &self.routing_key,
                BasicPublishOptions::default(),
                &payload,
                BasicProperties::default(),
            )
            .await
            .expect("Failed to publish message");

        log::info!("Email relayed to RabbitMQ: {:?}", eml_data.message_id);
    }
}
