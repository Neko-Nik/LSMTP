use crate::handler::amqp::start_amqp_publisher;
use super::types::{Email, BaseConfig};
use tokio::net::TcpListener;


// Type alias for the email sender channel
pub type EmailSender = tokio::sync::mpsc::Sender<Email>;


// Temporary email storage directory if the AMQP publish fails
const TMP_EMAIL_DIR: &str = "/tmp/lsmtp";


/// Initializes the Logging, TCP Listener, and AMQP Publisher for the LSMTP Daemon
pub async fn init() -> (TcpListener, EmailSender) {
    // Initialize logging
    env_logger::init();

    // Create temporary email storage directory if it doesn't exist
    std::fs::create_dir_all(TMP_EMAIL_DIR).unwrap();

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
