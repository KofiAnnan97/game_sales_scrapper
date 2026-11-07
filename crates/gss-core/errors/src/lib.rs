// use crate::alert::AlertError;
// use crate::api::ApiError;
// use crate::config::ConfigError;
// use crate::file::FileOperationError;
// use crate::secrets::SecretsError;
// use std::fmt::{Display, Formatter};

pub mod alert;
pub mod api;
pub mod config;
pub mod file;
pub mod secrets;

// #[derive(Debug)]
// pub enum GenericError {
//     File(FileOperationError),
//     Config(ConfigError),
//     Secrets(SecretsError),
//     Api(ApiError),
//     Alert(AlertError),
//     Message(String),
// }

// impl From<FileOperationError> for GenericError {
//     fn from(err: FileOperationError) -> Self {
//         GenericError::File(err)
//     }
// }

// impl From<ConfigError> for GenericError {
//     fn from(err: ConfigError) -> Self {
//         GenericError::Config(err)
//     }
// }

// impl From<SecretsError> for GenericError {
//     fn from(err: SecretsError) -> Self {
//         GenericError::Secrets(err)
//     }
// }

// impl From<ApiError> for GenericError {
//     fn from(err: ApiError) -> Self {
//         GenericError::Api(err)
//     }
// }

// impl From<AlertError> for GenericError {
//     fn from(err: AlertError) -> Self {
//         GenericError::Alert(err)
//     }
// }

// impl From<String> for GenericError {
//     fn from(value: String) -> Self {
//         GenericError::Message(value)
//     }
// }

// impl Display for GenericError {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         match self {
//             GenericError::Alert(e) => write!(f, "An alert error occured. {}", e),
//             GenericError::Api(e) => write!(f, "An external service error occurred. {}", e),
//             GenericError::Config(e) => write!(f, "A config error occurred. {}", e),
//             GenericError::File(e) => write!(f, "A file operation error occurred. {}", e),
//             GenericError::Secrets(e) => write!(f, "A secrets error occurred. {}", e),
//             GenericError::Message(msg) => write!(f, "A generic error occurred. {}", msg),
//         }
//     }
// }
