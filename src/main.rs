mod smtp_client;

use tokio::net::TcpListener;
use tokio::fs::{OpenOptions, File};
use std::sync::Arc;
use tokio::sync::Mutex;
use self::smtp_client::handle_client;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:25").await?;
    println!("Neko Nik - LSMTP Daemon started on port 25");

    // Shared log file (thread-safe)
    let log_file: Arc<Mutex<File>> = Arc::new(Mutex::new(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open("/var/log/lsmtpd.log")
            .await?,
    ));

    loop {
        let (socket, _) = listener.accept().await?;

        // Get client IP address
        let client_addr: SocketAddr = socket.peer_addr()?;
        println!("Incoming connection from: {}", client_addr);

        let log_file = log_file.clone();

        tokio::spawn(async move {
            if let Err(err) = handle_client(socket, log_file, client_addr).await {
                eprintln!("Error handling client from {}: {:?}", client_addr, err);
            }
        });
    }
}
