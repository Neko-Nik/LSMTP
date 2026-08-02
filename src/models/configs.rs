use std::env::var as env_var;
use std::sync::LazyLock;


// ------- Structs ------- //


pub struct AMQPConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
    vhost: String,
    exchange: String,
    routing_key: String,
    pub buffer_size: usize,
}


pub struct BaseConfig {
    bind_address: String,
    bind_port: u16,
    pub amqp_details: AMQPConfig,
}


// ------- Static Variables ------- //


// Name of the server, used only at greeting, HELO, EHLO commands
pub static SERVER_NAME: LazyLock<String> = LazyLock::new(|| {
    // Read the SERVER_NAME environment variable and panic if it's not set
    env_var("SERVER_NAME").expect("SERVER_NAME must be set")
});


// Maximum email size that the server will accept
pub static MAX_EMAIL_SIZE_BYTES: LazyLock<usize> = LazyLock::new(|| {
    // Read the MAX_EMAIL_SIZE_BYTES environment variable
    // and panic if it's not set or cannot be parsed as a usize
    env_var("MAX_EMAIL_SIZE_BYTES")
        .expect("MAX_EMAIL_SIZE_BYTES must be set to a valid usize")
        .parse::<usize>()
        .expect("MAX_EMAIL_SIZE_BYTES must be set to a valid usize")
});


// Temporary email storage directory if the AMQP publish fails
pub static TEMP_EMAIL_DIR: LazyLock<String> = LazyLock::new(|| {
    // Read the TEMP_EMAIL_DIR environment variable and panic if it's not set
    env_var("TEMP_EMAIL_DIR").expect("TEMP_EMAIL_DIR must be set to a valid directory path")
});


// Maximum timeout for client connections in seconds
pub static MAX_TIMEOUT_SECS: LazyLock<u64> = LazyLock::new(|| {
    // Read the MAX_TIMEOUT_SECS environment variable
    // and panic if it's not set or cannot be parsed as a u64
    env_var("MAX_TIMEOUT_SECS")
        .expect("MAX_TIMEOUT_SECS must be set to a valid u64")
        .parse::<u64>()
        .expect("MAX_TIMEOUT_SECS must be set to a valid u64")
});


// ------- Implementations ------- //


impl BaseConfig {
    /// Reads configuration from environment variables and returns a BaseConfig instance.
    pub fn from_env() -> Self {
        let bind_address = env_var("BIND_ADDRESS")
            .expect("BIND_ADDRESS must be set to a valid IP address or hostname");
        let bind_port = env_var("BIND_PORT")
            .expect("BIND_PORT must be set to a valid u16")
            .parse::<u16>()
            .expect("BIND_PORT must be set to a valid u16");

        let amqp_host = env_var("AMQP_HOST")
            .expect("AMQP_HOST must be set");
        let amqp_port = env_var("AMQP_PORT")
            .expect("AMQP_PORT must be set to a valid u16")
            .parse::<u16>()
            .expect("AMQP_PORT must be set to a valid u16");
        let amqp_username = env_var("AMQP_USERNAME")
            .expect("AMQP_USERNAME must be set");
        let amqp_password = env_var("AMQP_PASSWORD")
            .expect("AMQP_PASSWORD must be set");
        let amqp_vhost = env_var("AMQP_VHOST")
            .expect("AMQP_VHOST must be set");
        let amqp_exchange = env_var("AMQP_EXCHANGE")
            .expect("AMQP_EXCHANGE must be set");
        let amqp_routing_key = env_var("AMQP_ROUTING_KEY")
            .expect("AMQP_ROUTING_KEY must be set");
        let amqp_buffer_size = env_var("AMQP_BUFFER_SIZE")
            .expect("AMQP_BUFFER_SIZE must be set to a valid usize")
            .parse::<usize>()
            .expect("AMQP_BUFFER_SIZE must be set to a valid usize");

        log::info!("All environment variables have been loaded");
        let amqp_details = AMQPConfig {
            host: amqp_host,
            port: amqp_port,
            username: amqp_username,
            password: amqp_password,
            vhost: amqp_vhost,
            exchange: amqp_exchange,
            routing_key: amqp_routing_key,
            buffer_size: amqp_buffer_size,
        };

        BaseConfig {
            bind_address,
            bind_port,
            amqp_details,
        }
    }

    /// Returns the bind URI in the format "address:port".
    pub fn bind_uri(&self) -> String {
        format!("{}:{}", self.bind_address, self.bind_port)
    }
}


impl AMQPConfig {
    /// Constructs the AMQP URL in the format "amqp://username:password@host:port/vhost".
    pub fn amqp_url(&self) -> String {
        format!(
            "amqp://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.vhost
        )
    }

    /// Returns the exchange name.
    pub fn exchange(&self) -> String {
        self.exchange.clone()
    }

    /// Returns the routing key.
    pub fn routing_key(&self) -> String {
        self.routing_key.clone()
    }
}
