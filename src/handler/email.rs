use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use super::parsing::{SMTPCommand, SMTPResponse};
use tokio::net::TcpStream;
use crate::types::Email;


// TODO: Try to add support for TLS / SSL / STARTTLS
pub async fn handle_client(socket: TcpStream, server_name: String) -> Result<Email, std::io::Error> {
    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);
    let mut buffer = String::new();
    let mut eml_data = Email::empty();
    let mut data_mode: bool = false;

    // Greet the client with the server's welcome message
    writer.write_all(&SMTPResponse::greet(&server_name)).await?;

    // Main loop to handle SMTP commands
    loop {
        buffer.clear();
        let bytes_read = reader.read_line(&mut buffer).await?;
        if bytes_read == 0 {
            break;  // TODO: Why? Test it once maybe because the client disconnected
        }

        let command: &str = buffer.trim_end();

        if data_mode {
            if command == "." { // End of DATA
                writer.write_all(&SMTPResponse::OK_WITH_MESSAGE_RESPONSE).await?;
                data_mode = false; // TODO: Should i break the loop ? (Test it once)
            } else {
                eml_data.add_content(format!("{}\n", command));
            }
        } else {
            match SMTPCommand::from_str(command) {

                SMTPCommand::HELO => {
                    eml_data.set_client_address(command[5..].trim().to_string());
                    writer.write_all(&SMTPResponse::helo_response(&server_name)).await?;
                }

                SMTPCommand::EHLO => {
                    eml_data.set_client_address(command[5..].trim().to_string());
                    writer.write_all(&SMTPResponse::ehlo_response(&server_name)).await?;
                }

                SMTPCommand::MailFrom => {
                    eml_data.set_sender(command[10..].trim().to_string());
                    writer.write_all(&SMTPResponse::OK_RESPONSE).await?;
                }

                SMTPCommand::RcptTo => {
                    eml_data.add_recipient(command[8..].trim().to_string());
                    writer.write_all(&SMTPResponse::OK_RESPONSE).await?;
                }

                SMTPCommand::Data => {
                    writer.write_all(&SMTPResponse::DATA_RESPONSE).await?;
                    data_mode = true;
                }

                SMTPCommand::Quit => {
                    writer.write_all(&SMTPResponse::BYE_RESPONSE).await?;
                    break;
                }

                SMTPCommand::Unknown => {
                    writer.write_all(&SMTPResponse::NOT_IMPLEMENTED_RESPONSE).await?;
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
