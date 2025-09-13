mod prelude;
mod handler;
mod types;
mod state;
mod amqp;


#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let (listener, amqp_tx, host_name) = state::init().await;

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                log::trace!("Incoming connection from: {}", addr);

                let server_name = host_name.clone();
                let ref_amqp_tx = prelude::arc_clone(&amqp_tx);

                tokio::spawn(async move {
                    match handler::email::handle_client(socket, server_name).await {
                        Ok(email) => {
                            log::info!("Received email: {:?}", email.get_id());

                            // Send to AMQP
                            if let Err(e) = ref_amqp_tx.send(email).await {
                                log::error!("Failed to send email to AMQP channel: {}", e);
                            }
                        },
                        Err(e) => {
                            log::error!("Error handling client: {}", e);
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
