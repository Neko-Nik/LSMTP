use super::prelude::{mpsc, sleep, Duration, BasicPublishOptions, Connection, ConnectionProperties, BasicProperties};
use super::types::{AMQPConfig, Email};


// Temporary email storage directory if the AMQP publish fails
const TMP_EMAIL_DIR: &str = "/tmp/lsmtp";


/// Locally save the email to a path
fn save_email_locally(email: &Email) {
    let path = format!("{}/{}.json", TMP_EMAIL_DIR, email.get_id());

    // Warn the user that we are using a temporary storage location
    log::warn!("Saving email to temporary location, manual intervention required: {}", path);

    // Write the email to the file system
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

        let mut channel = match connection.create_channel().await {
            Ok(ch) => ch,
            Err(e) => { log::error!("create_channel failed: {}", e); return; }
        };

        while let Some(email) = rx.recv().await {
            let msg_id: &str = email.get_id();
            log::debug!("Publishing email to AMQP: {}", msg_id);

            // Check if the channel is still open, recreate if needed
            channel = if !channel.status().connected() {
                log::warn!("AMQP channel disconnected, recreating channel for email: {}", msg_id);
                match connection.create_channel().await {
                    Ok(ch) => ch,
                    Err(e) => {
                        log::error!("Failed to recreate channel: {:?} for email: {}", e, msg_id);
                        save_email_locally(&email);
                        continue;
                    }
                }
            } else {
                channel
            };

            // publish the message
            match channel.basic_publish(
                &amqp_config.exchange(),
                &amqp_config.routing_key(),
                BasicPublishOptions::default(),
                &email.serialize(),
                BasicProperties::default(),
            ).await {
                Ok(confirm) => {
                    if let Err(e) = confirm.await {
                        log::error!("AMQP publish not confirmed: {:?} for email: {}", e, msg_id);
                        save_email_locally(&email);
                    }
                    log::trace!("AMQP publish confirmed for email: {}", msg_id);
                }
                Err(e) => {
                    log::error!("AMQP publish failed: {:?} for email: {}", e, msg_id);
                    save_email_locally(&email);
                }
            }
        }

        log::info!("AMQP publisher exiting; sender closed");
    });

    tx
}
