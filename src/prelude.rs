pub use lapin::{BasicProperties, Connection, ConnectionProperties, options::BasicPublishOptions};
pub use tokio::time::{sleep, Duration};
pub use std::env::var as env_var;
pub use tokio::net::TcpListener;
pub use log::{info, error};
pub use tokio::sync::mpsc;
pub use serde::Serialize;
pub use std::sync::Arc;
pub use serde_json;


pub fn uuid_v4() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn current_timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}

pub fn arc_clone<T>(value: &Arc<T>) -> Arc<T> {
    Arc::clone(value)
}
