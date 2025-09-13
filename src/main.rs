mod prelude;
mod handler;
mod types;
mod state;
mod amqp;


#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    // Initialize the application state
    let (listener, amqp_tx, host_name) = state::init().await;

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                log::trace!("Incoming connection from: {}", addr);

                // Clone the server name and AMQP sender reference
                let server_name = host_name.clone();
                let amqp_txn = amqp_tx.clone();

                // Spawn a new task to handle the client connection
                tokio::spawn(async move {
                    // Create a new email handler
                    let client = handler::email::EmailHandler::new(socket);

                    // Run the client with a 3 minute timeout
                    match prelude::timeout(prelude::Duration::from_secs(180), client.run(server_name)).await {
                        Ok(run_result) => {
                            // client.run completed before timeout, now inspect result
                            match run_result {
                                Ok(email) => {
                                    log::info!("Received email: {}", email.get_id());

                                    // Send email to AMQP channel to process with backpressure (channel buffering)
                                    if let Err(e) = amqp_txn.send(email).await {
                                        log::error!("Failed to send email to AMQP channel: {}", e);
                                    }
                                }
                                Err(e) => {
                                    log::error!("Error handling client: {}", e);
                                }
                            }
                        }

                        // Timeout elapsed
                        Err(elapsed) => {
                            log::warn!("Connection handler timed out after 180s: dropping connection, timeout error details: {:?}", elapsed);
                        }
                    }
                });
            },
            Err(e) => {
                log::error!("Error accepting connection: {:?}", e);
            }
        }
    }
}
