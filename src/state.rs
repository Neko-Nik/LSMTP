use super::prelude::*;
use super::types::*;
use super::amqp::*;


pub async fn init() -> (TcpListener, Arc<std::sync::mpsc::Sender<Email>>, String) {
    env_logger::init();

    // Preparing to start the server by collecting environment variables
    let base_config = BaseConfig::from_env();
    let host_name = base_config.host_name();

    // Initialize the TCP listener
    let listener = TcpListener::bind(base_config.bind_uri())
        .await
        .expect("Failed to bind to address");

    info!("LSMTP Daemon started on {}", base_config.bind_uri());

    // Initialize the channel
    let (tx, rx) = std::sync::mpsc::channel::<Email>();
    amqp_process_channel(base_config.amqp_details, rx).await;

    (listener, Arc::new(tx), host_name)
}
