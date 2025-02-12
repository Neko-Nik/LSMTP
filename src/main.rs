use utils::client::handle_client;
use utils::base::get_env_vars;
use tokio::net::TcpListener;
use std::sync::Arc;

mod utils;


#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    env_logger::init();
    let config = Arc::new(get_env_vars().expect("Failed to get configuration variables"));

    let listener = TcpListener::bind(format!("{}:{}", config.bind_address, config.bind_port)).await?;
    log::info!("Neko Nik - LSMTP Daemon started on port 25");

    loop {
        match listener.accept().await {
            Ok((_socket, addr)) => {
                log::trace!("Incoming connection from: {}", addr);

                // Clone the configuration for each client
                let config_clone = Arc::clone(&config);

                tokio::spawn(async move {
                    if let Err(err) = handle_client(_socket, config_clone).await {
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
