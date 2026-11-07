use crate::file::FileOperationError;
use crate::secrets::SecretsError;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum ConfigError {
    FileError(FileOperationError),
    SecretError(SecretsError),
    Message(String),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::FileError(e) => write!(f, "{}", e),
            ConfigError::SecretError(e) => write!(f, "{}", e),
            ConfigError::Message(msg) => write!(f, "Config Error - {}", msg),
        }
    }
}
