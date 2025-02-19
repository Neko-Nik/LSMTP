use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use super::amqp::AMQPClient;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::io::Result;
use serde::Serialize;
use std::sync::Arc;
use chrono::Local;
use uuid::Uuid;


#[derive(Debug, Serialize)]
pub struct EMLData {
    timestamp: String,
    pub message_id: String,
    recipients: Vec<String>,
    email_content: String,
    sender: String,
}

enum SMTPCommand {
    EHLO,
    HELO,
    MailFrom,
    RcptTo,
    DATA,
    QUIT,
    UNKNOWN,
}

impl SMTPCommand {
    fn from_str(command: &str) -> Self {
        let command_upper = command.to_uppercase();
        if command_upper.starts_with("EHLO") {
            SMTPCommand::EHLO
        } else if command_upper.starts_with("HELO") {
            SMTPCommand::HELO
        } else if command_upper.starts_with("MAIL FROM:") {
            SMTPCommand::MailFrom
        } else if command_upper.starts_with("RCPT TO:") {
            SMTPCommand::RcptTo
        } else if command_upper == "DATA" {
            SMTPCommand::DATA
        } else if command_upper == "QUIT" {
            SMTPCommand::QUIT
        } else {
            SMTPCommand::UNKNOWN
        }
    }
}


enum SMTPResponse {
    OK,
    DATA,
    BYE,
    NotImplemented,
    OkWithMessage,
    WelcomeMessage,
    HeloResponse,
}

impl SMTPResponse {
    fn as_bytes(&self) -> &'static [u8] {
        match self {
            SMTPResponse::OK => b"250 OK\r\n",
            SMTPResponse::DATA => b"354 End data with <CR><LF>.<CR><LF>\r\n",
            SMTPResponse::BYE => b"221 Bye\r\n",
            SMTPResponse::NotImplemented => b"502 Command not implemented\r\n",
            SMTPResponse::OkWithMessage => b"250 OK: Message accepted\r\n",
            SMTPResponse::WelcomeMessage => b"220 Neko Nik LSMTP Server (Debian/GNU)\r\n",
            SMTPResponse::HeloResponse => b"250 Neko Nik LSMTP Server\r\n",
        }
    }
}


pub async fn handle_client(socket: TcpStream, server_name: String, amqp_client: Arc<Mutex<AMQPClient>>) -> Result<()> {
    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);
    let mut buffer = String::new();
    let mut eml_data = EMLData {
        sender: String::new(),
        recipients: Vec::new(),
        email_content: String::new(),
        timestamp: Local::now().to_string(),
        message_id: Uuid::new_v4().to_string(),
    };
    let mut data_mode: bool = false;

    // Start the SMTP conversation
    writer.write_all(SMTPResponse::WelcomeMessage.as_bytes()).await?;

    // Main loop to handle SMTP commands
    loop {
        buffer.clear();
        let bytes_read = reader.read_line(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }

        let command: &str = buffer.trim_end();

        if data_mode {
            if command == "." { // End of DATA
                writer.write_all(SMTPResponse::OkWithMessage.as_bytes()).await?;
                break;
            } else {
                eml_data.email_content.push_str(&format!("{}\n", command));
            }
        } else {
            match SMTPCommand::from_str(command) {

                SMTPCommand::EHLO | SMTPCommand::HELO => {
                    writer.write_all(SMTPResponse::HeloResponse.as_bytes()).await?;
                }

                SMTPCommand::MailFrom => {
                    eml_data.sender = command[10..].trim().to_string();
                    writer.write_all(SMTPResponse::OK.as_bytes()).await?;
                }

                SMTPCommand::RcptTo => {
                    eml_data.recipients.push(command[8..].trim().to_string());
                    writer.write_all(SMTPResponse::OK.as_bytes()).await?;
                }

                SMTPCommand::DATA => {
                    writer.write_all(SMTPResponse::DATA.as_bytes()).await?;
                    eml_data.email_content.push_str(&format!("Message-ID: <{}@{}>\n", eml_data.message_id, server_name));
                    data_mode = true;
                }

                SMTPCommand::QUIT => {
                    writer.write_all(SMTPResponse::BYE.as_bytes()).await?;
                    break;
                }

                SMTPCommand::UNKNOWN => {
                    writer.write_all(SMTPResponse::NotImplemented.as_bytes()).await?;
                }

            }
        }
    }

    if eml_data.recipients.is_empty() || eml_data.sender.is_empty() || eml_data.email_content.is_empty() {
        log::warn!("Invalid email data, either sender, recipients or email content is empty: {:?}", eml_data);
        return Ok(());
    }

    amqp_client.lock().await.publish(&eml_data).await;

    Ok(())
}
