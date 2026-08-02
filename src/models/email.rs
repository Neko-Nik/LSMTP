use super::configs::{SERVER_NAME, MAX_EMAIL_SIZE_BYTES};
use serde::Serialize;


// ------- Enums ------- //


pub enum SMTPCommand {
    EHLO,       // Extended HELO
    HELO,       // Hello

    MailFrom,   // Mail From
    RcptTo,     // Recipient To

    Data,       // Email Raw Data
    Dot,        // End of data

    Quit,       // Close connection
    Noop,       // No operation
    Reset,      // Reset all
    Unknown,    // Unknown
}


pub enum SMTPResponse {
    Ok,                 // 250 OK
    Bye,                // 221 Bye
    Data,               // 354 End data with <CR><LF>.<CR><LF>
    NotImplemented,     // 502 Command not implemented
    SizeExceeded,       // 552 Message size exceeds fixed maximum message size

    Greet,              // 220 <server> LSMTP Server (Rust)
    Helo,               // 250 <server>
    Ehlo,               // 250-<server>  250-SIZE <max_size>  250-8BITMIME  250 OK

    DataEnd(String),    // 250 2.0.0 Ok: queued as <message_id>
}


// ------- Structs ------- //


#[derive(Serialize)]
pub struct Email {
    timestamp: String,
    pub message_id: String,
    client_address: String,
    recipients: Vec<String>,
    email_content: Vec<u8>,
    sender: String,
}


#[derive(Serialize)]
struct EmailPayload {
    timestamp: String,
    message_id: String,
    client_address: String,
    recipients: Vec<String>,
    email_content: String,
    sender: String,
}


// ------- Implementations ------- //


impl Email {
    pub fn new(msg_id: uuid::Uuid) -> Self {
        Email {
            timestamp: chrono::Utc::now().to_rfc3339(),
            message_id: msg_id.to_string(),
            recipients: Vec::new(),
            email_content: Vec::new(),
            client_address: String::new(),
            sender: String::new(),
        }
    }

    pub fn reset(&mut self) {
        self.email_content.clear();
        self.recipients.clear();
        self.sender.clear();
    }

    pub fn set_client_address(&mut self, client_address: String) {
        self.client_address = client_address;
    }

    pub fn add_recipient(&mut self, recipient: String) {
        self.recipients.push(recipient);
    }

    pub fn add_content(&mut self, content: Vec<u8>) {
        self.email_content.extend_from_slice(&content);
    }

    pub fn set_sender(&mut self, sender: String) {
        self.sender = sender;
    }

    pub fn serialize(&self) -> Vec<u8> {
        let payload = EmailPayload {
            timestamp: self.timestamp.clone(),
            message_id: self.message_id.clone(),
            client_address: self.client_address.clone(),
            recipients: self.recipients.clone(),
            email_content: String::from_utf8_lossy(&self.email_content).into_owned(),
            sender: self.sender.clone(),
        };

        serde_json::to_vec(&payload).expect("Failed to serialize Email")
    }

    pub fn validate(&self) -> Result<(), &str> {
        // TODO: Add more validation checks as needed along with some good email validation's
        if self.sender.is_empty() {
            return Err("Sender is empty");
        }
        if self.recipients.is_empty() {
            return Err("Recipients are empty");
        }
        if self.email_content.is_empty() {
            return Err("Email content is empty");
        }
        Ok(())
    }
}


impl SMTPCommand {
    pub fn from_str(command: &str) -> Self {
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
            SMTPCommand::Data
        } else if command_upper == "." {
            SMTPCommand::Dot
        } else if command_upper == "RSET" {
            SMTPCommand::Reset
        } else if command_upper == "NOOP" {
            SMTPCommand::Noop
        } else if command_upper == "QUIT" {
            SMTPCommand::Quit
        } else {
            SMTPCommand::Unknown
        }
    }
}


impl SMTPResponse {
    pub fn mail_from_response(addr_part: &str, max_email_size: usize) -> (String, bool) {
        let mut sender = String::new();
        let mut valid = true;
        let parts = addr_part.split_whitespace().collect::<Vec<&str>>();

        for part in parts {
            let upper = part.to_uppercase();

            if upper.starts_with("SIZE=") {
                if let Ok(size) = part[5..].parse::<usize>() {
                    if size >= max_email_size {
                        valid = false;
                    }
                }
            } else if upper.starts_with("BODY=") {
                let body = upper.trim_start_matches("BODY=");
                if body != "7BIT" && body != "8BITMIME" {
                    valid = false;
                    break;
                }
            } else if part.starts_with('<') && part.ends_with('>') {
                sender = part[1..part.len()-1].to_string();
            } else if part.contains('@') {
                // In some cases, the address may be specified without angle brackets
                sender = part.to_string();
            } else {
                // Invalid address format
                log::warn!("Invalid MAIL FROM address format: {}", addr_part);
                valid = false;
                break;
            }
        }

        (sender, valid)
    }


    fn ehlo_response() -> Vec<u8> {
        // Note that the last response should not have "-" at the beginning
        // But the top level responses should
        // Example 1: [250 OK] (that is end)
        // Example 2: [250-TEST  250-SIZE  250-PARAMETER  250 EndCMD] (as you can see end will not have "-" at the beginning)
        let mut response = format!("250-{}\r\n", SERVER_NAME.to_string());

        response.push_str(format!("250-SIZE {}\r\n", *MAX_EMAIL_SIZE_BYTES).as_str());
        response.push_str("250-8BITMIME\r\n");
        // response.push_str("250-PIPELINING\r\n");
        // response.push_str("250-ENHANCEDSTATUSCODES\r\n");
        // response.push_str("250 STARTTLS\r\n");
        // response.push_str("250-SMTPUTF8\r\n");
        // response.push_str("250 CHUNKING\r\n");
        // response.push_str("250 DSN\r\n");
        // response.push_str("250 VRFY\r\n");
        // response.push_str("250 ETRN\r\n");
        response.push_str("250 OK\r\n");

        response.into_bytes()
    }


    pub fn into_bytes(&self) -> Vec<u8> {
        match self {
            SMTPResponse::Ok => b"250 OK\r\n".to_vec(),
            SMTPResponse::Bye => b"221 Bye\r\n".to_vec(),
            SMTPResponse::Data => b"354 End data with <CR><LF>.<CR><LF>\r\n".to_vec(),
            SMTPResponse::NotImplemented => b"502 Command not implemented\r\n".to_vec(),
            SMTPResponse::SizeExceeded => b"552 Message size exceeds fixed maximum message size\r\n".to_vec(),
            SMTPResponse::Greet => format!("220 {} LSMTP Server (Rust)\r\n", SERVER_NAME.to_string()).into_bytes(),
            SMTPResponse::Helo => format!("250 {}\r\n", SERVER_NAME.to_string()).into_bytes(),
            SMTPResponse::Ehlo => Self::ehlo_response(),
            SMTPResponse::DataEnd(message_id) => format!("250 Ok: queued as {}\r\n", message_id).into_bytes(),
        }
    }
}
