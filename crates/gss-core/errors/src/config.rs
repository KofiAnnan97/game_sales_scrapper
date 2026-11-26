use std::env::VarError;
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::input::{FileOperationError, UserInputError};
use crate::secrets::SecretsError;
#[derive(Debug)]
pub enum ConfigError {
    File(FileOperationError),
    Secret(SecretsError),
    Environ(VarError),
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

impl From<VarError> for ConfigError {
    fn from(err: VarError) -> Self {
        ConfigError::Environ(err)
    }
}

impl From<UserInputError> for ConfigError {
    fn from(err: UserInputError) -> Self {
        ConfigError::Message(err.to_string())
    }
}

impl Error for ConfigError {}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::File(e) => write!(f, "{}", e),
            ConfigError::Secret(e) => write!(f, "{}", e),
            ConfigError::Environ(e) => write!(f, "{}", e),
            ConfigError::Message(msg) => write!(f, "Config Error - {}", msg),
        }
    }
}
