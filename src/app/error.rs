use std::error;
use std::fmt;
use log::SetLoggerError;
use noritama::error::Error as NoritamaError;

// @TODO Implement ErrorKind

#[derive(Debug)]
pub enum Error {
    InvalidArgError(String),
    LibError(NoritamaError),
    LoggerError(SetLoggerError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::InvalidArgError(ref err) => write!(f, "InvalidError: {}", err),
            Error::LibError(ref err) => write!(f, "LibError: {}", err),
            Error::LoggerError(ref err) => write!(f, "LoggerError: {}", err),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::InvalidArgError(ref err) => err,
            Error::LibError(ref err) => err.description(),
            Error::LoggerError(ref err) => err.description(),
        }
    }
}

impl From<NoritamaError> for Error {
    fn from(err: NoritamaError) -> Error {
        Error::LibError(err)
    }
}

impl From<SetLoggerError> for Error {
    fn from(err: SetLoggerError) -> Error {
        Error::LoggerError(err)
    }
}
