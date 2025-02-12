use utils::client::handle_client;
use utils::base::get_env_vars;
use tokio::net::TcpListener;
use utils::amqp::AMQPClient;
use tokio::sync::Mutex;
use std::sync::Arc;

mod utils;


#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    env_logger::init();
    let config = Arc::new(get_env_vars().expect("Failed to get configuration variables"));
    let amqp_client = Arc::new(Mutex::new(AMQPClient::new(&config.amqp_details).await));
    let listener = TcpListener::bind(format!("{}:{}", config.bind_address, config.bind_port)).await?;
    log::info!("Neko Nik - LSMTP Daemon started on port {}", config.bind_port);

    loop {
        match listener.accept().await {
            Ok((_socket, addr)) => {
                log::trace!("Incoming connection from: {}", addr);

                // Clone the AMQP client and server name for each client
                let server_name: String = config.server_name.clone();
                let amqp_client_clone = Arc::clone(&amqp_client);

                tokio::spawn(async move {
                    if let Err(err) = handle_client(_socket, server_name, amqp_client_clone).await {
                        log::error!("Error handling client from {}: {:?}", addr, err);
                    } else {
                        log::trace!("Client from {} disconnected", addr);
                    }
                });
            },
            Err(e) => {
                log::error!("Error accepting connection: {:?}", e);
            }
        }
    }
}
