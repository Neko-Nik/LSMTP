use std::env;
use std::error::Error;


#[derive(Debug)]
pub struct AMQPConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub vhost: String,
    pub exchange: String,
    pub routing_key: String,
}

pub struct EnvVars {
    pub bind_address: String,
    pub bind_port: u16,
    pub server_name: String,
    pub amqp_details: AMQPConfig,
}


pub fn get_env_vars() -> Result<EnvVars, Box<dyn Error>> {
    let bind_address = env::var("BIND_ADDRESS")?;
    let bind_port = env::var("BIND_PORT")?.parse::<u16>()?;
    let server_name = env::var("SERVER_NAME")?;

    let amqp_host = env::var("AMQP_HOST")?;
    let amqp_port = env::var("AMQP_PORT")?.parse::<u16>()?;
    let amqp_username = env::var("AMQP_USERNAME")?;
    let amqp_password = env::var("AMQP_PASSWORD")?;
    let amqp_vhost = env::var("AMQP_VHOST")?;
    let amqp_exchange = env::var("AMQP_EXCHANGE")?;
    let amqp_routing_key = env::var("AMQP_ROUTING_KEY")?;

    let amqp_details = AMQPConfig {
        host: amqp_host,
        port: amqp_port,
        username: amqp_username,
        password: amqp_password,
        vhost: amqp_vhost,
        exchange: amqp_exchange,
        routing_key: amqp_routing_key,
    };

    log::debug!("Environment variables loaded successfully");

    Ok(EnvVars {
        bind_address,
        bind_port,
        server_name,
        amqp_details,
    })
}
