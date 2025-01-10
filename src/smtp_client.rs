use chrono::Local;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::Result;
use tokio::sync::Mutex;
use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};


fn add_to_header(email_content: &mut String, all_recipients: &Vec<String>) {
    // Since we are using a mutable reference, we can modify the email_content directly
    // X-Real-To means the actual recipients of the email which includes the BCC recipients as well
    let real_to_header: String = format!("X-Real-To: {}", all_recipients.join(", ").replace("<", "").replace(">", ""));
    *email_content = format!("{}\n{}", real_to_header, email_content);  // Update email_content
}


pub async fn handle_client(socket: tokio::net::TcpStream, log_file: Arc<Mutex<File>>, client_addr: SocketAddr) -> Result<()> {
    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);
    let mut buffer = String::new();

    // Logging session
    let mut session_log = String::new();

    // TODO: Add proper SMTP server banner
    writer.write_all(b"220 Neko Nik LSMTP Server (Debian/GNU)\r\n").await?;
    session_log.push_str("220 Neko Nik LSMTP Server (Debian/GNU)\n");

    let mut sender = String::new();
    let mut recipients = Vec::new();
    let mut data_mode = false;
    let mut email_content = String::new();

    dbg!("Starting to read from client:", &client_addr);

    // TODO: Add checks for each command
    // - Check if before DATA, there is a valid sender and at least one recipient
    // - Check if after DATA, there is a valid email content
    // - Check if the email content is valid (RFC 5322) [Optional]
    loop {
        buffer.clear();
        let bytes_read = reader.read_line(&mut buffer).await?;
        if bytes_read == 0 {
            dbg!("Connection closed");
            break; // Connection closed
        }

        session_log.push_str(&format!("> {}", buffer));
        let command = buffer.trim_end();

        if data_mode {
            if command == "." {
                // End of DATA
                writer.write_all(b"250 OK: Message accepted\r\n").await?;
                session_log.push_str("250 OK: Message accepted\n");
                dbg!("End of DATA");
                break;
            } else {
                email_content.push_str(&format!("{}\n", command));
            }
        } else {
            match command {
                command if command.starts_with("EHLO") || command.starts_with("HELO") => {
                    dbg!("EHLO/HELO", &command);
                    writer.write_all(b"250 Neko-Nik-Relay\r\n").await?;
                    session_log.push_str("250 Neko-Nik-Relay\n");
                }
                command if command.starts_with("MAIL FROM:") => {
                    dbg!("MAIL FROM");
                    sender = command[10..].trim().to_string();
                    writer.write_all(b"250 OK\r\n").await?;
                    session_log.push_str("250 OK\n");
                }
                command if command.starts_with("RCPT TO:") => {
                    dbg!("RCPT TO");
                    recipients.push(command[8..].trim().to_string());
                    writer.write_all(b"250 OK\r\n").await?;
                    session_log.push_str("250 OK\n");
                }
                "DATA" => {
                    dbg!("DATA");
                    writer.write_all(b"354 End data with <CR><LF>.<CR><LF>\r\n").await?;
                    session_log.push_str("354 End data with <CR><LF>.<CR><LF>\n");
                    data_mode = true;
                }
                "QUIT" => {
                    dbg!("QUIT");
                    writer.write_all(b"221 Bye\r\n").await?;
                    session_log.push_str("221 Bye\n");
                    break;
                }
                _ => {
                    dbg!("Command not implemented", &command);
                    writer.write_all(b"502 Command not implemented\r\n").await?;
                    session_log.push_str("502 Command not implemented\n");
                }
            }
        }
    }

    add_to_header(&mut email_content, &recipients);

    relay_it_to_amqp(&sender, &recipients, &email_content).await?;

    // Log the session
    // TODO: Use log library
    let log_entry = format!(
        "Timestamp: {}\nSession Log:\n{}\nSender: {}\nRecipients: {:?}\nEmail Content:\n{}\n--------------------------\n",
        Local::now(),
        session_log,
        sender,
        recipients,
        email_content
    );

    let mut log_file = log_file.lock().await;
    log_file.write_all(log_entry.as_bytes()).await?;

    dbg!("Session logged");

    Ok(())
}



async fn relay_it_to_amqp(actual_sender: &str, all_recipients: &Vec<String>, email_content: &str) -> tokio::io::Result<()> {
    // TODO: Connect to AMQP server and send the data ?
    dbg!("Relaying email to AMQP server");

    dbg!("Sender:", actual_sender);
    dbg!("Recipients:", all_recipients);
    dbg!("Email Content:", email_content);

    dbg!("Email sent to real SMTP server");
    Ok(())
}
