pub use tokio::net::TcpListener;
pub use serde::{Serialize, Deserialize};
pub use serde_json;
pub use std::env::var as env_var;
pub use std::sync::mpsc::Receiver;
pub use lapin::{BasicProperties, Connection, ConnectionProperties, Channel, options::BasicPublishOptions};
pub use std::time::Duration;
pub use log::{info, error};
pub use std::error::Error;
pub use tokio::sync::Mutex;
pub use std::sync::Arc;


pub fn uuid_v4() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn current_timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}

pub fn arc_clone<T>(value: &Arc<T>) -> Arc<T> {
    Arc::clone(value)
}
