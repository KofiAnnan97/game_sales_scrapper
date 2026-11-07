use std::fmt::{Display, Formatter};
use std::io;

#[derive(Debug)]
pub enum FileOperationError {
    CSVError(csv::Error),
    IOError(io::Error),
    Message(String),
}

impl Display for FileOperationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FileOperationError::CSVError(e) => write!(f, "CSV Error - {}", e),
            FileOperationError::IOError(e) => write!(f, "IO Error - {}", e),
            FileOperationError::Message(msg) => write!(f, "File Operation Error - {}", msg),
        }
    }
}
