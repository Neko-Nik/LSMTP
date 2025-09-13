use super::prelude::{TcpListener, Arc};
use super::types::{Email, BaseConfig};
use super::amqp::start_amqp_publisher;


pub async fn init() -> (TcpListener, Arc<tokio::sync::mpsc::Sender<Email>>, String) {
    env_logger::init();

    // Preparing to start the server by collecting environment variables
    let base_config = BaseConfig::from_env();
    let host_name = base_config.host_name();

    // Initialize the TCP listener
    let listener = TcpListener::bind(base_config.bind_uri())
        .await
        .expect("Failed to bind to address");

    log::info!("LSMTP Daemon started on {}", base_config.bind_uri());

    // Initialize the channel
    let tx = start_amqp_publisher(base_config.amqp_details);

    (listener, Arc::new(tx), host_name)
}
