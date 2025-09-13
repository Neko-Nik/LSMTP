pub use lapin::{BasicProperties, Connection, ConnectionProperties, options::BasicPublishOptions};
pub use tokio::time::{sleep, Duration};
pub use std::env::var as env_var;
pub use tokio::net::TcpListener;
pub use tokio::sync::mpsc;
pub use serde::Serialize;


pub fn uuid_v4() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn current_timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}
