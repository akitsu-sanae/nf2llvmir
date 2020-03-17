use super::*;
use std::error;

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

impl From<typecheck::Error> for Error {
    fn from(err: typecheck::Error) -> Self {
        Error::Typecheck(err)
    }
}

impl From<codegen::Error> for Error {
    fn from(err: codegen::Error) -> Self {
        Error::Codegen(err)
    }
}
