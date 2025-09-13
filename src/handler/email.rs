use tokio::net::{TcpStream, tcp::{OwnedReadHalf, OwnedWriteHalf}};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use super::parsing::{SMTPCommand, SMTPResponse};
use crate::types::Email;


/// Per-connection client object that owns the reader/writer and session state.
pub struct EmailHandler {
    reader: BufReader<OwnedReadHalf>,
    writer: OwnedWriteHalf,
    email: Email,
    data_mode: bool,
    buffer: String,
}


impl EmailHandler {
    /// Create a EmailHandler from a connected TcpStream
    pub fn new(socket: TcpStream) -> Self {
        let (read_half, write_half) = socket.into_split();
        EmailHandler {
            reader: BufReader::new(read_half),
            writer: write_half,
            email: Email::empty(),
            data_mode: false,
            buffer: String::with_capacity(1024),
        }
    }

    /// Run the client session. Consumes self and returns the Email (or IO error).
    pub async fn run(mut self, server_name: String) -> Result<Email, std::io::Error> {
        // greet the client
        self.writer.write_all(&SMTPResponse::greet(&server_name)).await?;

        loop {
            self.buffer.clear();
            let bytes_read = self.reader.read_line(&mut self.buffer).await?;
            if bytes_read == 0 {
                // peer closed connection
                self.writer.shutdown().await?;
                break;
            }

            // trim only CRLF, keep content for parsing
            let line = self.buffer.trim_end_matches(&['\r', '\n'][..]);

            if self.data_mode {
                if line == "." {
                    // end of DATA
                    self.writer.write_all(&SMTPResponse::OK_WITH_MESSAGE_RESPONSE).await?;
                    self.data_mode = false;
                    // Typically the DATA end completes a message; we can break to return Email
                    // break;
                } else {
                    // append data line to email content (dot-stuffing not handled here if needed)
                    self.email.add_content(format!("{}\n", line));
                    continue;
                }
            }

            // Not in data mode — parse command
            match SMTPCommand::from_str(line) {
                SMTPCommand::HELO => {
                    // safely get argument after command: avoid direct slicing
                    let arg = line.get(5..).unwrap_or("").trim().to_string();
                    self.email.set_client_address(arg);
                    self.writer.write_all(&SMTPResponse::helo_response(&server_name)).await?;
                }

                SMTPCommand::EHLO => {
                    let arg = line.get(5..).unwrap_or("").trim().to_string();
                    self.email.set_client_address(arg);
                    self.writer.write_all(&SMTPResponse::ehlo_response(&server_name)).await?;
                }

                SMTPCommand::MailFrom => {
                    // safe slice: MAIL FROM: is 10 chars, but use get to avoid panic
                    let arg = line.get(10..).unwrap_or("").trim().to_string();
                    self.email.set_sender(arg);
                    self.writer.write_all(&SMTPResponse::OK_RESPONSE).await?;
                }

                SMTPCommand::RcptTo => {
                    let arg = line.get(8..).unwrap_or("").trim().to_string();
                    self.email.add_recipient(arg);
                    self.writer.write_all(&SMTPResponse::OK_RESPONSE).await?;
                }

                SMTPCommand::Data => {
                    self.writer.write_all(&SMTPResponse::DATA_RESPONSE).await?;
                    self.data_mode = true;
                }

                SMTPCommand::Quit => {
                    self.writer.write_all(&SMTPResponse::BYE_RESPONSE).await?;
                    self.writer.shutdown().await?;
                    break;
                }

                SMTPCommand::Unknown => {
                    self.writer.write_all(&SMTPResponse::NOT_IMPLEMENTED_RESPONSE).await?;
                }
            }
        }

        // Final validation
        match self.email.validate() {
            Ok(_) => Ok(self.email),
            Err(e) => {
                log::warn!("Invalid email data: {}", e);
                self.writer.shutdown().await?;
                Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid email data"))
            }
        }
    }
}
