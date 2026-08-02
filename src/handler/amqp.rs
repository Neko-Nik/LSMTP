use lapin::{BasicProperties, Connection, ConnectionProperties, options::BasicPublishOptions};
use crate::models::configs::AMQPConfig;
use crate::state::save_local_email;
use tokio::time::{sleep, Duration};
use crate::models::email::Email;
use tokio::sync::mpsc;


struct AMQP {
    connection: Connection,
    channel: lapin::Channel,
}


impl AMQP {
    fn connected(&self) -> bool {
        self.connection.status().connected()
            && self.channel.status().connected()
    }

    async fn publish(&self, config: &AMQPConfig, payload: &[u8]) -> Result<(), lapin::Error> {
        let confirm = self.channel
            .basic_publish(
                &config.exchange(),
                &config.routing_key(),
                BasicPublishOptions::default(),
                payload,
                BasicProperties::default(),
            )
            .await?;

        confirm.await?;

        Ok(())
    }
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

        log::info!("Attempting to connect to AMQP (try {})", retries + 1);
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
        sleep(Duration::from_secs(3)).await;    // Wait before retrying
    }
}


/// Attempt to reconnect to AMQP, closing any existing connection first
async fn reconnect(amqp: &mut Option<AMQP>, config: &AMQPConfig) {
    // If we have an existing connection, close it before reconnecting
    if let Some(old) = amqp.take() {
        log::warn!("Closing old AMQP connection for reconnect");
        close_amqp(old).await;
    }

    // Attempt to do a new connection
    log::info!("Reconnecting to AMQP");
    *amqp = connect_amqp(config).await;
}


/// Ensure we have a live AMQP connection, reconnecting if necessary
async fn ensure_connection(amqp: &mut Option<AMQP>, cfg: &AMQPConfig) {
    // If we already have a live connection, do nothing
    if amqp
        .as_ref()
        .is_some_and(|a| a.connected())
    {
        // Connection is alive, nothing to do
        return;
    }

    // If we don't have a live connection, log a warning and attempt to reconnect
    log::warn!("AMQP connection lost, reconnecting!");
    reconnect(amqp, cfg).await;
}


/// Start the AMQP publisher task and return a sender for sending emails to be published
pub fn start_amqp_publisher(amqp_config: AMQPConfig) -> mpsc::Sender<Email> {
    let (tx, mut rx) = mpsc::channel::<Email>(amqp_config.buffer_size);

    tokio::spawn(async move {
        // connect with retry
        let mut amqp: Option<AMQP> = connect_amqp(&amqp_config).await;

        while let Some(email) = rx.recv().await {
            let msg_id = &email.message_id;
            let email_bytes = email.serialize();
            log::debug!("Publishing email to AMQP: {}", msg_id);

            // Ensure we have a live connection
            ensure_connection(&mut amqp, &amqp_config).await;

            // If we still don't have a connection, save the email locally and continue
            let Some(active) = &amqp else {
                save_local_email(msg_id, &email_bytes);
                continue;
            };

            // Attempt to publish the email
            if let Err(e) = active.publish(&amqp_config, &email_bytes).await {
                log::error!("AMQP publish failed: {:?} for email: {}", e, msg_id);
                save_local_email(msg_id, &email_bytes);
                reconnect(&mut amqp, &amqp_config).await;
            } else {
                log::trace!("AMQP publish confirmed for email: {}", msg_id);
            }
        }

        // The sender has been closed, exit the publisher task
        amqp.map(|a| tokio::spawn(close_amqp(a)));

        log::info!("AMQP publisher exiting; sender closed");
    });

    tx
}
