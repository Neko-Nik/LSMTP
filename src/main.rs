mod handler;
mod models;
mod errors;
mod queue;
mod state;


#[tokio::main]
async fn main() -> Result<(), errors::LSMTPError> {
    // Initialize the application state
    let (listener, amqp_tx) = state::init().await;
    log::debug!("Configuration loaded. Listening for incoming connections");

    loop {
        // Accept all and any incoming connections
        let (socket, addr) = listener.accept().await?;
        log::trace!("Incoming connection from: {}", addr);

        // Clone the AMQP sender reference
        let amqp_tx = amqp_tx.clone();

        // Spawn a new task to handle the client connection
        tokio::spawn(async move {
            state::handle_connection(socket, addr, amqp_tx).await;
        });
    }
}
