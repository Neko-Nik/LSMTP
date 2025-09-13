use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use super::parsing::{SMTPCommand, SMTPResponse};
use tokio::net::TcpStream;
use crate::types::Email;


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
                    writer.write_all(&SMTPResponse::Ok.as_bytes(None)).await?;
                }

                SMTPCommand::RcptTo => {
                    // eml_data.recipients.push(command[8..].trim().to_string());
                    eml_data.add_recipient(command[8..].trim().to_string());
                    writer.write_all(&SMTPResponse::Ok.as_bytes(None)).await?;
                }

                SMTPCommand::Data => {
                    writer.write_all(&SMTPResponse::Data.as_bytes(None)).await?;
                    // eml_data.email_content.push_str(&format!("Message-ID: <{}@{}>\n", eml_data.message_id, server_name));
                    data_mode = true;
                }

                SMTPCommand::Quit => {
                    writer.write_all(&SMTPResponse::Bye.as_bytes(None)).await?;
                    break;
                }

                SMTPCommand::Unknown => {
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
