use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::file::FileOperationError;
use crate::secrets::SecretsError;
#[derive(Debug)]
pub enum ConfigError {
    File(FileOperationError),
    Secret(SecretsError),
    Message(String),
}

impl From<FileOperationError> for ConfigError {
    fn from(err: FileOperationError) -> Self {
        ConfigError::File(err)
    }
}

impl From<SecretsError> for ConfigError {
    fn from(err: SecretsError) -> Self {
        ConfigError::Secret(err)
    }
}

impl Error for ConfigError {}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::File(e) => write!(f, "{}", e),
            ConfigError::Secret(e) => write!(f, "{}", e),
            ConfigError::Message(msg) => write!(f, "Config Error - {}", msg),
        }
    }
}
