use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use crate::types::Email;


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
    fn as_bytes(&self, server_name: Option<&str>) -> Vec<u8> {
        match self {
            SMTPResponse::OK => b"250 OK\r\n".to_vec(),
            SMTPResponse::DATA => b"354 End data with <CR><LF>.<CR><LF>\r\n".to_vec(),
            SMTPResponse::BYE => b"221 Bye\r\n".to_vec(),
            SMTPResponse::NotImplemented => b"502 Command not implemented\r\n".to_vec(),
            SMTPResponse::OkWithMessage => b"250 OK: Message accepted\r\n".to_vec(),
            SMTPResponse::WelcomeMessage => format!("220 {} LSMTP Server (Rust)\r\n", server_name.unwrap_or("Neko Nik")).into_bytes(),
            SMTPResponse::HeloResponse => format!("250 {}\r\n", server_name.unwrap_or("Neko Nik")).into_bytes(),
        }
    }
}


pub async fn handle_client(socket: TcpStream, server_name: String) -> Result<Email, std::io::Error> {
    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);
    let mut buffer = String::new();
    let mut eml_data = Email::empty();
    let mut data_mode: bool = false;

    // TODO: Try to add support for TLS / SSL / STARTTLS

    // Start the SMTP conversation
    writer.write_all(&SMTPResponse::WelcomeMessage.as_bytes(Some(&server_name))).await?;

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
                writer.write_all(&SMTPResponse::OkWithMessage.as_bytes(None)).await?;
                break;
            } else {
                // eml_data.email_content.push_str(&format!("{}\n", command));
                eml_data.add_content(format!("{}\n", command));
            }
        } else {
            match SMTPCommand::from_str(command) {

                SMTPCommand::EHLO | SMTPCommand::HELO => {
                    writer.write_all(&SMTPResponse::HeloResponse.as_bytes(Some(&server_name))).await?;
                }

                SMTPCommand::MailFrom => {
                    // eml_data.sender = command[10..].trim().to_string();
                    eml_data.set_sender(command[10..].trim().to_string());
                    writer.write_all(&SMTPResponse::OK.as_bytes(None)).await?;
                }

                SMTPCommand::RcptTo => {
                    // eml_data.recipients.push(command[8..].trim().to_string());
                    eml_data.add_recipient(command[8..].trim().to_string());
                    writer.write_all(&SMTPResponse::OK.as_bytes(None)).await?;
                }

                SMTPCommand::DATA => {
                    writer.write_all(&SMTPResponse::DATA.as_bytes(None)).await?;
                    // eml_data.email_content.push_str(&format!("Message-ID: <{}@{}>\n", eml_data.message_id, server_name));
                    data_mode = true;
                }

                SMTPCommand::QUIT => {
                    writer.write_all(&SMTPResponse::BYE.as_bytes(None)).await?;
                    break;
                }

                SMTPCommand::UNKNOWN => {
                    writer.write_all(&SMTPResponse::NotImplemented.as_bytes(None)).await?;
                }

            }
        }
    }

    match eml_data.validate() {
        Ok(_) => {},
        Err(e) => {
            log::warn!("Invalid email data: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid email data"));
        }
    }

    Ok(eml_data)
}
