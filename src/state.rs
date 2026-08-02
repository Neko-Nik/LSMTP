use crate::models::configs::{BaseConfig, MAX_TIMEOUT_SECS, TEMP_EMAIL_DIR};
use tokio::net::{TcpListener, TcpStream};
use crate::handler::email::EmailHandler;
use crate::queue::start_amqp_publisher;
use crate::models::email::Email;
use std::net::SocketAddr;
use tokio::time;


// Type alias for the email sender channel
pub type EmailSender = tokio::sync::mpsc::Sender<Email>;


/// Initializes the Logging, TCP Listener, and AMQP Publisher for the LSMTP Daemon
pub async fn init() -> (TcpListener, EmailSender) {
    // Initialize logging
    env_logger::init();

    // Create temporary email storage directory if it doesn't exist
    std::fs::create_dir_all(TEMP_EMAIL_DIR.as_str()).unwrap();

    // Preparing to start the server by collecting environment variables
    let base_config = BaseConfig::from_env();

    // Initialize the TCP listener
    let listener = TcpListener::bind(base_config.bind_uri())
        .await
        .expect("Failed to bind to address");

    log::info!("LSMTP Daemon started on {}", base_config.bind_uri());

    // Initialize the channel
    let tx = start_amqp_publisher(base_config.amqp_details);

    (listener, tx)
}


/// Handle a single client connection. This function is spawned as a new task for each connection.
/// This is the main logic for handling a client connection
pub async fn handle_connection(socket: TcpStream, addr: SocketAddr, amqp_tx: EmailSender) {
    // Create a new UUID for the connection/session
    let conn_id = uuid::Uuid::new_v4();

    // Create a new email handler
    let client = EmailHandler::new(socket, conn_id);

    log::debug!("[conn={}] Handling connection from: {}", conn_id, addr);

    // Run the client with a timeout
    match time::timeout(time::Duration::from_secs(*MAX_TIMEOUT_SECS), client.run()).await {
        Ok(Ok(email)) => {
            log::info!("[conn={}] Received email: {}", conn_id, email.message_id);

            // Send the email to the AMQP channel
            if let Err(e) = amqp_tx.send(email).await {
                log::error!("[conn={}] Failed to send email to AMQP channel: {}", conn_id, e);
            }
        }

        Ok(Err(e)) => {
            log::error!("[conn={}] Error handling client {}: {}", conn_id, addr, e);
        }

        Err(_) => {
            log::warn!("[conn={}] Connection handler timed out after {} seconds for client: {}", conn_id, *MAX_TIMEOUT_SECS, addr);
        }
    }
}


/// Locally save the contents of an email to a temporary directory for later retrieval or manual intervention
pub fn save_local_email(message_id: &str, contents: &[u8]) {
    let path = format!("{}/{}.json", TEMP_EMAIL_DIR.as_str(), message_id);

    // Warn the user that we are using a temporary storage location
    log::warn!("Saving email to temporary location, manual intervention required: {}", &path);

    // Write the email to the file system
    if let Err(e) = std::fs::write(&path, contents) {
        log::error!("Failed to save email locally and is totally lost! path: {}, error: {}", &path, e);
    }
}
