use reqwest;
use serde_json;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::write;

use crate::file::FileOperationError;

#[derive(Debug)]
pub enum ApiError {
    RequestError(reqwest::Error),
    JsonError(serde_json::Error),
    FileError(FileOperationError),
    Message(String),
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        ApiError::RequestError(err)
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::JsonError(err)
    }
}

impl From<FileOperationError> for ApiError {
    fn from(err: FileOperationError) -> Self {
        ApiError::FileError(err)
    }
}

impl Error for ApiError {}

impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::RequestError(e) => write!(f, "Request failed - {}", e),
            ApiError::JsonError(e) => write!(f, "JSON parsing failed - {}", e),
            ApiError::FileError(e) => write!(f, "IO failed - {}", e),
            ApiError::Message(msg) => write!(f, "{}", msg),
        }
    }
}
