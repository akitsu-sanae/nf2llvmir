use std::error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum Error {
    Internal(String),
    Validation(String),
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Internal(msg) => write!(f, "internal error: {}", msg),
            Error::Validation(msg) => write!(f, "validation error: {}", msg),
            Error::Io(err) => write!(f, "io error: {:?}", err),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}
