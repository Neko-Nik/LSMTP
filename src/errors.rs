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


impl Error for LSMTPError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            LSMTPError::IoError(e) => Some(e),
            // LSMTPError::AmqpError(e) => Some(e),
            _ => None,
        }
    }
}


impl From<std::io::Error> for LSMTPError {
    fn from(err: std::io::Error) -> Self {
        LSMTPError::IoError(err)
    }
}
