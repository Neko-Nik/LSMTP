use crate::state::EmailSender;
use tokio::net::{TcpStream};
use std::net::SocketAddr;
use tokio::time;


mod handler;
mod types;
mod state;
mod amqp;
mod errors;


const MAX_TIMEOUT_SECS: u64 = 180; // 3 minutes


/// Handle a single client connection. This function is spawned as a new task for each connection.
async fn handle_connection(socket: TcpStream, addr: SocketAddr, amqp_tx: EmailSender) {
    // Create a new UUID for the connection/session
    let conn_id = uuid::Uuid::new_v4();

    // Create a new email handler
    let client = handler::email::EmailHandler::new(socket, conn_id);

    log::debug!("[conn={}] Handling connection from: {}", conn_id, addr);

    // Run the client with a timeout
    match time::timeout(time::Duration::from_secs(MAX_TIMEOUT_SECS), client.run()).await {
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
            log::warn!("[conn={}] Connection handler timed out after {} seconds for client: {}", conn_id, MAX_TIMEOUT_SECS, addr);
        }
    }
}


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
            handle_connection(socket, addr, amqp_tx).await;
        });
    }
}
