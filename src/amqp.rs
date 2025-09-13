use super::prelude::{mpsc, sleep, Duration, BasicPublishOptions, Connection, ConnectionProperties, BasicProperties};
use super::types::{AMQPConfig, Email};


// Temporary email storage directory if the AMQP publish fails
const TMP_EMAIL_DIR: &str = "/tmp/lsmtp";


/// Locally save the email to a path
fn save_email_locally(email: &Email) {
    // Warn the user that we are using a temporary storage location
    log::warn!("Saving email to temporary location, need further processing, due to failed publish");

    // let path = format!("/tmp/lsmtp/{}.json", email.get_id());
    let path = format!("{}/{}.json", TMP_EMAIL_DIR, email.get_id());
    std::fs::write(path, email.serialize()).unwrap();
}


pub fn start_amqp_publisher(amqp_config: AMQPConfig) -> mpsc::Sender<Email> {
    let (tx, mut rx) = mpsc::channel::<Email>(amqp_config.buffer_size);

    tokio::spawn(async move {
        // connect with retry
        let connection = loop {
            match Connection::connect(&amqp_config.amqp_url(), ConnectionProperties::default()).await {
                Ok(c) => break c,
                Err(e) => {
                    log::error!("AMQP connect failed: {} - retrying in 3s", e);
                    sleep(Duration::from_secs(3)).await;
                }
            }
        };

        let channel = match connection.create_channel().await {
            Ok(ch) => ch,
            Err(e) => { log::error!("create_channel failed: {}", e); return; }
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
                log::error!("Publish to {} failed: {}", amqp_config.routing_key(), e);
                save_email_locally(&email);
            }
        }

        log::info!("AMQP publisher exiting; sender closed");
    });

    tx
}
