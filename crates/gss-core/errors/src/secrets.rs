use aes_gcm::aead;
use base64::DecodeError;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum DecryptError {
    Base64Error(base64::DecodeError),
    AesError(aead::Error),
    FormatError(FromUtf8Error),
    Message(String),
}

impl Error for DecryptError {}

impl From<base64::DecodeError> for DecryptError {
    fn from(err: DecodeError) -> Self {
        DecryptError::Base64Error(err)
    }
}

impl From<aead::Error> for DecryptError {
    fn from(err: aes_gcm::Error) -> Self {
        DecryptError::AesError(err)
    }
}

impl From<FromUtf8Error> for DecryptError {
    fn from(err: FromUtf8Error) -> Self {
        DecryptError::FormatError(err)
    }
}

impl Display for DecryptError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DecryptError::Base64Error(e) => write!(f, "Decrypt Error - {}", e),
            DecryptError::AesError(e) => write!(f, "Decrypt Error - {}", e),
            DecryptError::FormatError(e) => write!(f, "UTF8 Format Error - {}", e),
            DecryptError::Message(msg) => write!(f, "{}", msg),
        }
    }
}

#[derive(Debug)]
pub enum SecretsError {
    ParseError(io::Error),
    EncryptError(aead::Error),
    DecryptError(DecryptError),
    Message(String),
}

impl From<aead::Error> for SecretsError {
    fn from(err: aes_gcm::Error) -> Self {
        SecretsError::EncryptError(err)
    }
}

impl From<io::Error> for SecretsError {
    fn from(err: io::Error) -> Self {
        SecretsError::ParseError(err)
    }
}

impl From<DecryptError> for SecretsError {
    fn from(err: DecryptError) -> Self {
        SecretsError::DecryptError(err)
    }
}

impl Error for SecretsError {}

impl Display for SecretsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SecretsError::ParseError(e) => write!(f, "Parse Error - {}", e),
            SecretsError::EncryptError(e) => write!(f, "Encrypt Error - {}", e),
            SecretsError::DecryptError(e) => write!(f, "{}", e),
            SecretsError::Message(msg) => write!(f, "File Operation Error - {}", msg),
        }
    }
}
