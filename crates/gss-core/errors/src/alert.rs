use lettre::error;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum AlertError {
    General(error::Error),
    AddressError(lettre::address::AddressError),
    SmtpError(lettre::transport::smtp::Error),
    Message(String),
}

impl From<error::Error> for AlertError {
    fn from(err: error::Error) -> Self {
        AlertError::General(err)
    }
}

impl From<lettre::address::AddressError> for AlertError {
    fn from(err: lettre::address::AddressError) -> Self {
        AlertError::AddressError(err)
    }
}

impl From<lettre::transport::smtp::Error> for AlertError {
    fn from(err: lettre::transport::smtp::Error) -> Self {
        AlertError::SmtpError(err)
    }
}

impl Error for AlertError {}

impl Display for AlertError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertError::General(e) => write!(f, "General Failure - {}", e),
            AlertError::AddressError(e) => write!(f, "Email Address failure - {}", e),
            AlertError::SmtpError(e) => write!(f, "SMTP failure - {}", e),
            AlertError::Message(msg) => write!(f, "Alert Failure - {}", msg),
        }
    }
}
