use std::env;
use std::error::Error;


#[derive(Debug)]
pub struct RabbitConfig {
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
    pub rabbit_details: RabbitConfig,
}


pub fn get_env_vars() -> Result<EnvVars, Box<dyn Error>> {
    let bind_address = env::var("BIND_ADDRESS")?;
    let bind_port = env::var("BIND_PORT")?.parse::<u16>()?;
    let server_name = env::var("SERVER_NAME")?;

    let rabbit_host = env::var("RABBIT_HOST")?;
    let rabbit_port = env::var("RABBIT_PORT")?.parse::<u16>()?;
    let rabbit_username = env::var("RABBIT_USERNAME")?;
    let rabbit_password = env::var("RABBIT_PASSWORD")?;
    let rabbit_vhost = env::var("RABBIT_VHOST")?;
    let rabbit_exchange = env::var("RABBIT_EXCHANGE")?;
    let rabbit_routing_key = env::var("RABBIT_ROUTING_KEY")?;

    let rabbit_details = RabbitConfig {
        host: rabbit_host,
        port: rabbit_port,
        username: rabbit_username,
        password: rabbit_password,
        vhost: rabbit_vhost,
        exchange: rabbit_exchange,
        routing_key: rabbit_routing_key,
    };

    Ok(EnvVars {
        bind_address,
        bind_port,
        server_name,
        rabbit_details,
    })
}
