use crate::models::configs::AMQPConfig;
use crate::state::save_local_email;
use crate::models::email::Email;
use tokio::sync::mpsc;
use self::amqp::AMQP;


mod amqp;


/// Attempt to reconnect to AMQP, closing any existing connection first
async fn reconnect(amqp: &mut Option<AMQP>, config: &AMQPConfig) {
    // If we have an existing connection, close it before reconnecting
    if let Some(old) = amqp.take() {
        log::warn!("Closing old AMQP connection for reconnect");
        old.close().await;
    }

    // Attempt to do a new connection
    log::warn!("Reconnecting to AMQP");
    *amqp = AMQP::try_connect(config).await;
}


/// Ensure we have a live AMQP connection, reconnecting if necessary
async fn ensure_connection(amqp: &mut Option<AMQP>, cfg: &AMQPConfig) {
    // If we already have a live connection, do nothing
    if amqp
        .as_ref()
        .is_some_and(|a| a.is_connected())
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
    log::info!("Starting AMQP publisher task with buffer size: {}", amqp_config.buffer_size);

    tokio::spawn(async move {
        // connect with retry
        let mut amqp: Option<AMQP> = AMQP::try_connect(&amqp_config).await;
        log::debug!("AMQP publisher task started; initial connection: {}", amqp.as_ref().map_or("None".to_string(), |_| "Connected".to_string()));

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

                // Save the email locally and attempt to reconnect
                save_local_email(msg_id, &email_bytes);
                reconnect(&mut amqp, &amqp_config).await;
            } else {
                log::trace!("AMQP publish confirmed for email: {}", msg_id);
            }
        }

        // The sender has been closed, exit the publisher task
        amqp.as_ref().map(|a| a.close());

        log::info!("AMQP publisher exiting; sender closed");
    });

    tx
}
