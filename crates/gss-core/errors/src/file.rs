use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::Error as IOError;

#[derive(Debug)]
pub enum FileOperationError {
    CSV(csv::Error),
    JSON(serde_json::Error),
    IO(IOError),
    Message(String),
}

impl From<csv::Error> for FileOperationError {
    fn from(err: csv::Error) -> Self {
        FileOperationError::CSV(err)
    }
}

impl From<serde_json::Error> for FileOperationError {
    fn from(err: serde_json::Error) -> Self {
        FileOperationError::JSON(err)
    }
}

impl From<IOError> for FileOperationError {
    fn from(err: IOError) -> Self {
        FileOperationError::IO(err)
    }
}

impl Error for FileOperationError {}

impl Display for FileOperationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FileOperationError::CSV(e) => write!(f, "CSV Error - {}", e),
            FileOperationError::JSON(e) => write!(f, "JSON Error - {}", e),
            FileOperationError::IO(e) => write!(f, "IO Error - {}", e),
            FileOperationError::Message(msg) => write!(f, "File Operation Error - {}", msg),
        }
    }
}
