use super::prelude::{mpsc, sleep, Duration, BasicPublishOptions, Connection, ConnectionProperties, BasicProperties};
use super::types::{AMQPConfig, Email};


// Temporary email storage directory if the AMQP publish fails
const TMP_EMAIL_DIR: &str = "/tmp/lsmtp";


struct AMQP {
    connection: Connection,
    channel: lapin::Channel,
}


/// Locally save the email to a path
fn save_email_locally(email: &Email) {
    let path = format!("{}/{}.json", TMP_EMAIL_DIR, email.get_id());

    // Warn the user that we are using a temporary storage location
    log::warn!("Saving email to temporary location, manual intervention required: {}", path);

    // Write the email to the file system
    std::fs::write(path, email.serialize()).unwrap();
}


/// Cleanly tear down an AMQP connection
async fn close_amqp(amqp: AMQP) {
    let _ = amqp.channel.close(200, "reconnect").await;
    let _ = amqp.connection.close(200, "reconnect").await;
}


/// Establish a new AMQP connection + channel with retries
async fn connect_amqp(amqp_config: &AMQPConfig) -> Option<AMQP> {
    let mut retries = 0;

    loop {
        if retries >= 5 {
            log::error!("AMQP reconnect failed after {} retries", retries);
            return None;
        }

        match Connection::connect(
            &amqp_config.amqp_url(),
            ConnectionProperties::default(),
        )
        .await
        {
            Ok(connection) => match connection.create_channel().await {
                Ok(channel) => {
                    log::info!("Connected to AMQP");
                    return Some(AMQP { connection, channel });
                }
                Err(e) => {
                    log::error!("Failed to create AMQP channel: {}", e);
                }
            },
            Err(e) => {
                log::error!("Failed to connect to AMQP: {}", e);
            }
        }

        retries += 1;
        sleep(Duration::from_secs(3)).await;
    }
}


/// Start the AMQP publisher task and return a sender for sending emails to be published
pub fn start_amqp_publisher(amqp_config: AMQPConfig) -> mpsc::Sender<Email> {
    let (tx, mut rx) = mpsc::channel::<Email>(amqp_config.buffer_size);

    tokio::spawn(async move {
        // connect with retry
        let mut amqp: Option<AMQP> = connect_amqp(&amqp_config).await;

        while let Some(email) = rx.recv().await {
            let msg_id = email.get_id();
            log::debug!("Publishing email to AMQP: {}", msg_id);

            // Ensure we have a live connection
            let needs_reconnect: bool = match &amqp {
                Some(a) => {
                    !a.connection.status().connected()
                    || !a.channel.status().connected()
                }
                None => true,
            };

            if needs_reconnect {
                log::warn!("AMQP connection lost, reconnecting");
                if let Some(old) = amqp.take() {
                    close_amqp(old).await;
                }
                amqp = connect_amqp(&amqp_config).await;
            }

            let Some(active) = &amqp else {
                save_email_locally(&email);
                continue;
            };

            // Publish
            let publish = active.channel.basic_publish(
                &amqp_config.exchange(),
                &amqp_config.routing_key(),
                BasicPublishOptions::default(),
                &email.serialize(),
                BasicProperties::default(),
            ).await;

            match publish {
                Ok(confirm) => {
                    if let Err(e) = confirm.await {
                        log::error!("AMQP publish not confirmed: {:?} for email: {}", e, msg_id);
                        save_email_locally(&email);
                        if let Some(old) = amqp.take() {
                            close_amqp(old).await;
                        }
                        continue;
                    }
                    log::trace!("AMQP publish confirmed for email: {}", msg_id);
                }
                Err(e) => {
                    log::error!("AMQP publish failed: {:?} for {}", e, msg_id);
                    save_email_locally(&email);
                    if let Some(old) = amqp.take() {
                        close_amqp(old).await;
                    }
                }
            }
        }

        if let Some(old) = amqp {
            close_amqp(old).await;
        }

        log::info!("AMQP publisher exiting; sender closed");
    });

    tx
}
