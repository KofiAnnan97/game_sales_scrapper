use serde_json;
use reqwest;
use std::fmt::{Display, Formatter};
use std::error::Error;
use std::write;

#[derive(Debug)]
pub enum ApiError{
    RequestError(reqwest::Error),
    JsonError(serde_json::Error),
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

impl Error for ApiError{}

impl Display for ApiError{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::RequestError(e) => write!(f, "Request failed - {}", e),
            ApiError::JsonError(e) => write!(f, "JSON parsing failed - {}", e),
            ApiError::Message(msg) => write!(f, "{}", msg)
        }
    }
}

