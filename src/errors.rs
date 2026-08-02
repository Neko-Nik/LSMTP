use std::fmt;
use std::error::Error;


#[derive(Debug)]
pub enum LSMTPError {
    IoError(std::io::Error),
    // AmqpError(amqprs::error::Error),
    // TimeoutError,
    InvalidEmailFormat,
    // Other(String),
}


// ------- Implementations ------- //


impl fmt::Display for LSMTPError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LSMTPError::IoError(e) => write!(f, "I/O Error: {}", e),
            // LSMTPError::AmqpError(e) => write!(f, "AMQP Error: {}", e),
            // LSMTPError::TimeoutError => write!(f, "Operation timed out"),
            LSMTPError::InvalidEmailFormat => write!(f, "Invalid email format"),
            // LSMTPError::Other(msg) => write!(f, "{}", msg),
        }
    }
}


// TODO: Not required for now, but can be useful for future error handling and propagation
impl Error for LSMTPError {}


impl From<std::io::Error> for LSMTPError {
    fn from(err: std::io::Error) -> Self {
        LSMTPError::IoError(err)
    }
}
