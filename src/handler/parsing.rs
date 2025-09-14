pub enum SMTPCommand {
    EHLO,       // Extended HELO
    HELO,       // Hello

    MailFrom,   // Mail From
    RcptTo,     // Recipient To

    Data,       // Email Raw Data
    Dot,        // End of data

    Quit,       // Close connection
    Reset,      // Reset all
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
        } else if command_upper == "." {
            SMTPCommand::Dot
        } else if command_upper == "RSET" {
            SMTPCommand::Reset
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
    pub const SIZE_LIMIT_EXCEEDED_RESPONSE: &[u8] = b"552 Message size exceeds fixed maximum message size\r\n";

    pub fn greet(server_name: &String) -> Vec<u8> {
        format!("220 {} LSMTP Server (Rust)\r\n", server_name).into_bytes()
    }

    pub fn helo_response(server_name: &String) -> Vec<u8> {
        let response = format!("250 {}\r\n", server_name);
        response.into_bytes()
    }

    pub fn ehlo_response(server_name: &String, max_email_size: usize) -> Vec<u8> {
        // Note that last response should not have "-" at the beginning
        // But the top level responses should
        // Example 1: [250 OK] (that is end)
        // Example 2: [250-TEST  250-SIZE  250-PARAMETER  250 EndCMD] (as you can see end will not have "-" at the beginning)
        let mut response = format!("250-{}\r\n", server_name);

        response.push_str(format!("250 SIZE {}\r\n", max_email_size).as_str());
        // response.push_str("250-PIPELINING\r\n");
        // response.push_str("250-8BITMIME\r\n");
        // response.push_str("250-ENHANCEDSTATUSCODES\r\n");
        // response.push_str("250 STARTTLS\r\n");
        // response.push_str("250-SMTPUTF8\r\n");
        // response.push_str("250 CHUNKING\r\n");
        // response.push_str("250 DSN\r\n");
        // response.push_str("250 VRFY\r\n");
        // response.push_str("250 ETRN\r\n");

        response.into_bytes()
    }

    pub fn mail_from_response(addr_part: &str, max_email_size: usize) -> (String, bool) {
        let mut sender = String::new();
        let mut valid = true;
        let parts = addr_part.split_whitespace().collect::<Vec<&str>>();

        for part in parts {
            if part.starts_with("size=") {
                if let Ok(size) = part[5..].parse::<usize>() {
                    if size >= max_email_size {
                        valid = false;
                    }
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
}
