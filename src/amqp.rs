use super::prelude::*;
use super::types::*;


/// Process the channel
pub async fn amqp_process_channel(amqp_config: AMQPConfig, rx: Receiver<Email>) {
    // Start the AMQP connection
    let connection = Connection::connect(&amqp_config.amqp_url(), ConnectionProperties::default())
        .await
        .expect("Failed to connect to RabbitMQ");

    // Create or reuse the channel
    let channel = connection.create_channel()
        .await
        .expect("Failed to create channel");

    std::thread::spawn(async move || {
        // Keep reading
        loop {
            match rx.recv() {
                Ok(email) => {
                    log::info!("Received: {}", email.get_id());
                    
                    // Send it to queue
                    channel.basic_publish(
                        &amqp_config.exchange(),
                        &amqp_config.routing_key(),
                        BasicPublishOptions::default(),
                        &email.serialize(),
                        BasicProperties::default(),
                    )
                        .await
                        .expect("Failed to publish email");
                }

                Err(_) => {
                    log::error!("Failed to receive email");
                    break;
                }
            }
        }
    });
}
