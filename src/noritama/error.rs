use std::error;
use std::fmt;


// @TODO Implement ErrorKind
#[derive(Debug)]
pub enum Error {
    ParseError(String),
    InvalidArgError(String),
    IOError(String),
    InitError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ParseError(ref err) => write!(f, "{}", err),
            Error::InvalidArgError(ref err) => write!(f, "{}", err),
            Error::IOError(ref err) => write!(f, "{}", err),
            Error::InitError(ref err) => write!(f, "{}", err),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ParseError(ref err) => err,
            Error::InvalidArgError(ref err) => err,
            Error::IOError(ref err) => err,
            Error::InitError(ref err) => err,
        }
    }
}
