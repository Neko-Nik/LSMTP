use tokio::net::{TcpStream, tcp::{OwnedReadHalf, OwnedWriteHalf}};
use crate::models::configs::{SERVER_NAME, MAX_EMAIL_SIZE_BYTES};
use crate::models::email::{Email, SMTPCommand, SMTPResponse};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use crate::errors::LSMTPError;


/// Per-connection client object that owns the reader/writer and session state.
pub struct EmailHandler {
    connection_id: uuid::Uuid,
    reader: BufReader<OwnedReadHalf>,
    writer: OwnedWriteHalf,
    email: Email,
    data_mode: bool,
    buffer: Vec<u8>,
}


impl EmailHandler {
    /// Create a EmailHandler from a connected TcpStream
    pub fn new(socket: TcpStream, connection_id: uuid::Uuid) -> Self {
        let (read_half, write_half) = socket.into_split();
        let email_msg_id = uuid::Uuid::new_v4();

        log::info!("New connection established. Connection ID: {}, Email Message ID: {}", connection_id, email_msg_id);

        EmailHandler {
            connection_id,
            reader: BufReader::new(read_half),
            writer: write_half,
            email: Email::new(email_msg_id),
            data_mode: false,
            buffer: Vec::with_capacity(1024),
        }
    }

    async fn read_next_line(&mut self) -> Result<Option<(Vec<u8>, String)>, LSMTPError> {
        self.buffer.clear();
        let bytes_read = self.reader.read_until(b'\n', &mut self.buffer).await?;
        if bytes_read == 0 {
            return Ok(None);
        }

        let line_bytes = self.buffer
            .strip_suffix(b"\r\n")
            .or_else(|| self.buffer.strip_suffix(b"\n"))
            .unwrap_or(self.buffer.as_slice());

        let line = String::from_utf8_lossy(line_bytes).into_owned();

        Ok(Some((line_bytes.to_vec(), line)))
    }

    fn append_data_chunk(&mut self, line_bytes: &[u8]) {
        let mut body_line = line_bytes.to_vec();
        if body_line.starts_with(b".") {
            body_line.remove(0);
        }
        self.email.add_content(body_line);
        self.email.add_content(b"\r\n".to_vec());
    }

    /// Run the client session. Consumes self and returns the Email (or IO error).
    pub async fn run(mut self) -> Result<Email, LSMTPError> {
        // greet the client
        self.writer.write_all(&SMTPResponse::greet(&SERVER_NAME)).await?;

        loop {
            let Some((line_bytes, line)) = self.read_next_line().await? else {
                self.writer.shutdown().await?;
                break;
            };

            if self.data_mode {
                if line_bytes == b"." {
                    // end of DATA
                    self.data_mode = false;
                } else {
                    self.append_data_chunk(&line_bytes);
                    continue;
                }
            }

            // Not in data mode — parse command
            match SMTPCommand::from_str(&line) {
                SMTPCommand::HELO => {
                    // safely get argument after command: avoid direct slicing
                    let arg = line.get(5..).unwrap_or("").trim().to_string();
                    self.email.set_client_address(arg);
                    self.writer.write_all(&SMTPResponse::helo_response(&SERVER_NAME)).await?;
                }

                SMTPCommand::EHLO => {
                    let arg = line.get(5..).unwrap_or("").trim().to_string();
                    self.email.set_client_address(arg);
                    self.writer.write_all(&SMTPResponse::ehlo_response(&SERVER_NAME, *MAX_EMAIL_SIZE_BYTES)).await?;
                }

                SMTPCommand::MailFrom => {
                    // safe slice: MAIL FROM: is 10 chars, but use get to avoid panic
                    let addr_part = line.get(10..).unwrap_or("").trim();
                    let (sender, valid) = SMTPResponse::mail_from_response(addr_part, *MAX_EMAIL_SIZE_BYTES);
                    if !valid {
                        self.writer.write_all(&SMTPResponse::SIZE_LIMIT_EXCEEDED_RESPONSE).await?;
                        continue;
                    }
                    self.email.set_sender(sender);
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

                SMTPCommand::Noop => {
                    self.writer.write_all(&SMTPResponse::OK_RESPONSE).await?;
                }

                SMTPCommand::Dot => {
                    self.writer.write_all(&SMTPResponse::data_end_response(&self.email.message_id)).await?;
                    self.data_mode = false;

                    // We close the connection immediately after receiving the email data, as per typical SMTP behavior.
                    self.writer.shutdown().await?;
                    break;
                }

                SMTPCommand::Reset => {
                    self.email.reset();
                    self.buffer.clear();
                    self.data_mode = false;
                    self.writer.write_all(&SMTPResponse::OK_RESPONSE).await?;
                }

                SMTPCommand::Unknown => {
                    log::warn!("[conn={}] Received unknown command: {}", self.connection_id, line);
                    self.writer.write_all(&SMTPResponse::NOT_IMPLEMENTED_RESPONSE).await?;
                }
            }
        }

        // Final validation
        match self.email.validate() {
            Ok(_) => Ok(self.email),
            Err(e) => {
                log::warn!("[conn={}] Invalid email data: {}", self.connection_id, e);
                self.writer.shutdown().await?;
                Err(LSMTPError::InvalidEmailFormat)
            }
        }
    }
}
