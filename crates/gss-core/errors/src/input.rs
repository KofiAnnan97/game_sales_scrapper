use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::Error as IOError;
use std::num::{ParseFloatError, ParseIntError};

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
            FileOperationError::Message(msg) => write!(f, "{}", msg),
        }
    }
}

#[derive(Debug)]
pub enum UserInputError {
    IO(IOError),
    ParseInt(ParseIntError),
    ParseFloat(ParseFloatError),
}

impl From<IOError> for UserInputError {
    fn from(err: IOError) -> Self {
        UserInputError::IO(err)
    }
}

impl From<ParseIntError> for UserInputError {
    fn from(err: ParseIntError) -> Self {
        UserInputError::ParseInt(err)
    }
}

impl From<ParseFloatError> for UserInputError {
    fn from(err: ParseFloatError) -> Self {
        UserInputError::ParseFloat(err)
    }
}

impl Error for UserInputError {}

impl Display for UserInputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UserInputError::IO(e) => write!(f, "IO Error - {}", e),
            UserInputError::ParseInt(e) => write!(f, "ParseInt Error - {}", e),
            UserInputError::ParseFloat(e) => write!(f, "ParseFloat Error - {}", e),
        }
    }
}
