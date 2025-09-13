pub enum SMTPCommand {
    EHLO,       // Extended HELO
    HELO,       // Hello

    MailFrom,   // Mail From
    RcptTo,     // Recipient To

    Data,       // Data

    Quit,       // Quit
    Unknown,    // Unknown
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


// ------- Handle SMTP Commands ------- //

pub enum SMTPResponse {}

impl SMTPResponse {
    pub const OK_RESPONSE: &[u8] = b"250 OK\r\n";
    pub const DATA_RESPONSE: &[u8] = b"354 End data with <CR><LF>.<CR><LF>\r\n";
    pub const BYE_RESPONSE: &[u8] = b"221 Bye\r\n";
    pub const NOT_IMPLEMENTED_RESPONSE: &[u8] = b"502 Command not implemented\r\n";
    pub const OK_WITH_MESSAGE_RESPONSE: &[u8] = b"250 OK: Message accepted\r\n";

    pub fn greet(server_name: &String) -> Vec<u8> {
        format!("220 {} LSMTP Server (Rust)\r\n", server_name).into_bytes()
    }

    pub fn helo_response(server_name: &String) -> Vec<u8> {
        let response = format!("250-{}\r\n", server_name);
        response.into_bytes()
    }

    pub fn ehlo_response(server_name: &String) -> Vec<u8> {
        let mut response = format!("250-{}\r\n", server_name);

        response.push_str("250-SIZE 52428800\r\n");
        response.push_str("250-PIPELINING\r\n");
        response.push_str("250-8BITMIME\r\n");
        response.push_str("250-ENHANCEDSTATUSCODES\r\n");
        response.push_str("250 STARTTLS\r\n");
        response.push_str("250-SMTPUTF8\r\n");
        response.push_str("250 CHUNKING\r\n");
        response.push_str("250 DSN\r\n");
        response.push_str("250 VRFY\r\n");
        response.push_str("250 ETRN\r\n");

        response.into_bytes()
    }

}
