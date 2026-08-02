use tokio::net::{TcpStream, tcp::{OwnedReadHalf, OwnedWriteHalf}};
use crate::models::email::{Email, SMTPCommand, SMTPResponse};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use crate::models::configs::MAX_EMAIL_SIZE_BYTES;
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

    async fn read_next_line(&mut self) -> Result<Option<String>, LSMTPError> {
        self.buffer.clear();

        let bytes_read = self.reader.read_until(b'\n', &mut self.buffer).await?;

        if bytes_read == 0 {
            return Ok(None);
        }

        let line_bytes = self.buffer
            .strip_suffix(b"\r\n")
            .or_else(|| self.buffer.strip_suffix(b"\n"))
            .unwrap_or(self.buffer.as_slice());

        Ok(Some(String::from_utf8_lossy(line_bytes).into_owned()))
    }

    fn append_data_chunk(&mut self, line_bytes: &[u8]) {
        let bytes = if line_bytes.starts_with(b".") {
            &line_bytes[1..]
        } else {
            line_bytes
        };

        self.email.add_content(bytes);
        self.email.add_content(b"\r\n");
    }

    async fn reply(&mut self, response: SMTPResponse) -> Result<(), LSMTPError> {
        self.writer.write_all(&response.into_bytes()).await?;
        Ok(())
    }

    /// Run the client session. Consumes self and returns the Email (or IO error).
    pub async fn run(mut self) -> Result<Email, LSMTPError> {
        // greet the client
        self.reply(SMTPResponse::Greet).await?;

        loop {
            let Some(line) = self.read_next_line().await? else {
                self.writer.shutdown().await?;
                break;
            };
            let line_bytes = line.as_bytes();

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
                    self.reply(SMTPResponse::Helo).await?;
                }

                SMTPCommand::EHLO => {
                    let arg = line.get(5..).unwrap_or("").trim().to_string();
                    self.email.set_client_address(arg);
                    self.reply(SMTPResponse::Ehlo).await?;
                }

                SMTPCommand::MailFrom => {
                    // safe slice: MAIL FROM: is 10 chars, but use get to avoid panic
                    let addr_part = line.get(10..).unwrap_or("").trim();
                    let (sender, valid) = SMTPResponse::mail_from_response(addr_part, *MAX_EMAIL_SIZE_BYTES);
                    if !valid {
                        self.reply(SMTPResponse::SizeExceeded).await?;
                        continue;
                    }
                    self.email.set_sender(sender);
                    self.reply(SMTPResponse::Ok).await?;
                }

                SMTPCommand::RcptTo => {
                    let arg = line.get(8..).unwrap_or("").trim().to_string();
                    self.email.add_recipient(arg);
                    self.reply(SMTPResponse::Ok).await?;
                }

                SMTPCommand::Data => {
                    self.reply(SMTPResponse::Data).await?;
                    self.data_mode = true;
                }

                SMTPCommand::Quit => {
                    self.reply(SMTPResponse::Bye).await?;
                    self.writer.shutdown().await?;
                    break;
                }

                SMTPCommand::Noop => {
                    self.reply(SMTPResponse::Ok).await?;
                }

                SMTPCommand::Dot => {
                    self.reply(SMTPResponse::DataEnd(self.email.message_id.clone())).await?;
                    self.data_mode = false;

                    // We close the connection immediately after receiving the email data, as per typical SMTP behavior.
                    self.writer.shutdown().await?;
                    break;
                }

                SMTPCommand::Reset => {
                    self.email.reset();
                    self.buffer.clear();
                    self.data_mode = false;
                    self.reply(SMTPResponse::Ok).await?;
                }

                SMTPCommand::Unknown => {
                    log::warn!("[conn={}] Received unknown command: {}", self.connection_id, line);
                    self.reply(SMTPResponse::NotImplemented).await?;
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
