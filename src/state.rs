use super::types::{Email, BaseConfig, InternalConfig};
use super::amqp::start_amqp_publisher;
use super::prelude::TcpListener;


// Temporary email storage directory if the AMQP publish fails
const TMP_EMAIL_DIR: &str = "/tmp/lsmtp";


pub async fn init() -> (TcpListener, tokio::sync::mpsc::Sender<Email>, InternalConfig) {
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

    (listener, tx, base_config.internal)
}
