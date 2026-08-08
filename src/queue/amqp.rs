use lapin::{BasicProperties, Connection, ConnectionProperties, options::BasicPublishOptions};
use crate::models::configs::AMQPConfig;
use tokio::time::{sleep, Duration};


pub(super) struct AMQP {
    connection: Connection,
    channel: lapin::Channel,
}


impl AMQP {
    /// Establish a new AMQP connection and channel
    async fn connect(config: &AMQPConfig) -> Result<AMQP, lapin::Error> {
        let connection = Connection::connect(
            &config.amqp_url(),
            ConnectionProperties::default(),
        )
        .await?;

        let channel = connection.create_channel().await?;

        Ok(AMQP { connection, channel })
    }


    /// Attempt to establish a new AMQP connection and channel with retries
    pub async fn try_connect(config: &AMQPConfig) -> Option<AMQP> {
        for attempt in 1..=5 {
            match Self::connect(config).await {
                Ok(amqp) => return Some(amqp),
                Err(e) => {
                    log::error!("AMQP connection attempt {} failed: {}", attempt, e);
                    sleep(Duration::from_secs(3)).await;
                }
            }
        }
        None // Return None if all attempts fail
    }


    /// Check if the AMQP connection and channel are still connected
    pub fn is_connected(&self) -> bool {
        self.connection.status().connected() && self.channel.status().connected()
    }


    /// Cleanly tear down an AMQP connection
    pub async fn close(&self) {
        let _ = self.channel.close(200, "reconnect").await;
        let _ = self.connection.close(200, "reconnect").await;
    }


    /// Publish a message to the specified exchange and routing key
    pub async fn publish(&self, config: &AMQPConfig, payload: &[u8]) -> Result<(), lapin::Error> {
        let confirm = self.channel
            .basic_publish(
                &config.exchange,
                &config.routing_key,
                BasicPublishOptions::default(),
                payload,
                BasicProperties::default(),
            )
            .await?;

        confirm.await?;

        Ok(())
    }
}
