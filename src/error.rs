//! Implements Error type. It implements the `std::error::Error` for errors returned by ODBC and
//! allows for an error handling which his idomatic to rust

pub use safe::DiagRec;
use std::fmt::{Display, Formatter};
use std;

/// An ODBC Error
#[derive(Debug)]
pub enum Error {
    SqlError(DiagRec),
    EnvAllocFailure,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        use std::error::Error;
        write!(f, "{}", self.description())
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::SqlError(ref dr) => dr.message.as_str(),
            &Error::EnvAllocFailure => "Failed to allocate ODBC environment",
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}
