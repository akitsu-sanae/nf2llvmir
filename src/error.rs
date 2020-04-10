use std::error;
use std::fmt;

use crate::{codegen, typecheck};

#[derive(Debug)]
pub enum Error {
    Typecheck(typecheck::error::Error),
    Codegen(codegen::error::Error),
    Others(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            Typecheck(err) => write!(f, "{}", err),
            Codegen(err) => write!(f, "{}", err),
            Others(msg) => write!(f, "{}", msg),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use Error::*;
        match self {
            Typecheck(err) => Some(err),
            Codegen(err) => Some(err),
            Others(_) => None,
        }
    }
}

impl From<typecheck::error::Error> for Error {
    fn from(err: typecheck::error::Error) -> Self {
        Error::Typecheck(err)
    }
}

impl From<codegen::error::Error> for Error {
    fn from(err: codegen::error::Error) -> Self {
        Error::Codegen(err)
    }
}
