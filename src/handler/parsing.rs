pub enum SMTPCommand {
    EHLO,       // Extended HELO
    HELO,       // Hello

    MailFrom,   // Mail From
    RcptTo,     // Recipient To
    Data,       // Data
    Quit,       // Quit
    Unknown,    // Unknown
}

pub enum SMTPResponse {
    Ok,
    Data,
    Bye,
    NotImplemented,
    OkWithMessage,
    WelcomeMessage,
    HeloResponse,
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
        } else if command_upper == "QUIT" {
            SMTPCommand::Quit
        } else {
            SMTPCommand::Unknown
        }
    }
}

impl SMTPResponse {
    pub fn as_bytes(&self, server_name: Option<&str>) -> Vec<u8> {
        match self {
            SMTPResponse::Ok => b"250 OK\r\n".to_vec(),
            SMTPResponse::Data => b"354 End data with <CR><LF>.<CR><LF>\r\n".to_vec(),
            SMTPResponse::Bye => b"221 Bye\r\n".to_vec(),
            SMTPResponse::NotImplemented => b"502 Command not implemented\r\n".to_vec(),
            SMTPResponse::OkWithMessage => b"250 OK: Message accepted\r\n".to_vec(),
            SMTPResponse::WelcomeMessage => format!("220 {} LSMTP Server (Rust)\r\n", server_name.unwrap_or("Neko Nik")).into_bytes(),
            SMTPResponse::HeloResponse => format!("250 {}\r\n", server_name.unwrap_or("Neko Nik")).into_bytes(),
        }
    }
}
