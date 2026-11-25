use crate::input::FileOperationError;
use lettre::error;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum NotificationError {
    General(error::Error),
    AddressError(lettre::address::AddressError),
    SmtpError(lettre::transport::smtp::Error),
    Message(String),
}

impl From<error::Error> for NotificationError {
    fn from(err: error::Error) -> Self {
        NotificationError::General(err)
    }
}

impl From<lettre::address::AddressError> for NotificationError {
    fn from(err: lettre::address::AddressError) -> Self {
        NotificationError::AddressError(err)
    }
}

impl From<lettre::transport::smtp::Error> for NotificationError {
    fn from(err: lettre::transport::smtp::Error) -> Self {
        NotificationError::SmtpError(err)
    }
}

impl From<FileOperationError> for NotificationError {
    fn from(err: FileOperationError) -> Self {
        NotificationError::Message(err.to_string())
    }
}

impl Error for NotificationError {}

impl Display for NotificationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationError::General(e) => write!(f, "General Failure - {}", e),
            NotificationError::AddressError(e) => write!(f, "Email Address failure - {}", e),
            NotificationError::SmtpError(e) => write!(f, "SMTP failure - {}", e),
            NotificationError::Message(msg) => write!(f, "{}", msg),
        }
    }
}
