use super::prelude::*;


pub struct AMQPConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
    vhost: String,
    exchange: String,
    routing_key: String,
}

pub struct BaseConfig {
    bind_address: String,
    bind_port: u16,
    server_name: String,
    pub amqp_details: AMQPConfig,
}

#[derive(Debug, Serialize)]
pub struct Email {
    timestamp: String,
    message_id: String,
    recipients: Vec<String>,
    email_content: String,
    sender: String,
}


impl BaseConfig {
    pub fn from_env() -> Self {
        let bind_address = env_var("BIND_ADDRESS")
            .expect("BIND_ADDRESS must be set to a valid IP address or hostname");
        let bind_port = env_var("BIND_PORT")
            .expect("BIND_PORT must be set to a valid u16")
            .parse::<u16>()
            .expect("BIND_PORT must be set to a valid u16");
        let server_name = env_var("SERVER_NAME")
            .expect("SERVER_NAME must be set");

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

        info!("All environment variables have been loaded");
        let amqp_details = AMQPConfig {
            host: amqp_host,
            port: amqp_port,
            username: amqp_username,
            password: amqp_password,
            vhost: amqp_vhost,
            exchange: amqp_exchange,
            routing_key: amqp_routing_key,
        };

        BaseConfig {
            bind_address,
            bind_port,
            server_name,
            amqp_details,
        }
    }

    pub fn bind_uri(&self) -> String {
        format!("{}:{}", self.bind_address, self.bind_port)
    }

    pub fn host_name(&self) -> String {
        self.server_name.clone()
    }
}


impl AMQPConfig {
    pub fn amqp_url(&self) -> String {
        format!(
            "amqp://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.vhost
        )
    }

    pub fn exchange(&self) -> String {
        self.exchange.clone()
    }

    pub fn routing_key(&self) -> String {
        self.routing_key.clone()
    }
}


impl Email {
    pub fn new(recipients: Vec<String>, email_content: String, sender: String) -> Self {
        let timestamp = current_timestamp();
        let message_id = uuid_v4();
        Email {
            timestamp,
            message_id,
            recipients,
            email_content,
            sender,
        }
    }

    pub fn empty() -> Self {
        Email {
            timestamp: current_timestamp(),
            message_id: uuid_v4(),
            recipients: Vec::new(),
            email_content: String::new(),
            sender: String::new(),
        }
    }

    pub fn get_id(&self) -> &str {
        &self.message_id
    }

    pub fn add_recipient(&mut self, recipient: String) {
        self.recipients.push(recipient);
    }

    pub fn add_content(&mut self, content: String) {
        self.email_content.push_str(&content);
    }

    pub fn set_sender(&mut self, sender: String) {
        self.sender = sender;
    }

    pub fn serialize(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("Failed to serialize Email")
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.sender.is_empty() {
            return Err("Sender is empty".into());
        }
        if self.recipients.is_empty() {
            return Err("Recipients are empty".into());
        }
        if self.email_content.is_empty() {
            return Err("Email content is empty".into());
        }
        Ok(())
    }
}
