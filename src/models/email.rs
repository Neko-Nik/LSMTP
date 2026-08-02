use serde::Serialize;


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
