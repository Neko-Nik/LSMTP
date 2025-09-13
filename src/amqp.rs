use super::prelude::*;
use super::types::*;


pub fn start_amqp_publisher(amqp_config: AMQPConfig, buffer: usize) -> mpsc::Sender<Email> {
    let (tx, mut rx) = mpsc::channel::<Email>(buffer);

    tokio::spawn(async move {
        // connect with retry
        let connection = loop {
            match Connection::connect(&amqp_config.amqp_url(), ConnectionProperties::default()).await {
                Ok(c) => break c,
                Err(e) => {
                    error!("AMQP connect failed: {} - retrying in 3s", e);
                    sleep(Duration::from_secs(3)).await;
                }
            }
        };

        let channel = match connection.create_channel().await {
            Ok(ch) => ch,
            Err(e) => { error!("create_channel failed: {}", e); return; }
        };

        while let Some(email) = rx.recv().await {
            let payload = email.serialize();
            if let Err(e) = channel
                .basic_publish(
                    &amqp_config.exchange(),
                    &amqp_config.routing_key(),
                    BasicPublishOptions::default(),
                    &payload,
                    BasicProperties::default(),
                )
                .await
            {
                error!("Publish failed: {}", e);
                // TODO: implement retry / DLQ here
            }
        }

        info!("AMQP publisher exiting; sender closed");
    });

    tx
}
