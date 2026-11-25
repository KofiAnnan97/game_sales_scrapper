use reqwest;
use serde_json;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::write;

use crate::input::FileOperationError;

#[derive(Debug)]
pub enum ApiError {
    RequestError(reqwest::Error),
    PayloadError(serde_json::Error),
    Message(String),
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        ApiError::RequestError(err)
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::PayloadError(err)
    }
}

impl From<FileOperationError> for ApiError {
    fn from(err: FileOperationError) -> Self {
        ApiError::Message(err.to_string())
    }
}

impl Error for ApiError {}

impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::RequestError(e) => write!(f, "Request failed - {}", e),
            ApiError::PayloadError(e) => write!(f, "JSON parsing failed - {}", e),
            ApiError::Message(msg) => write!(f, "{}", msg),
        }
    }
}
