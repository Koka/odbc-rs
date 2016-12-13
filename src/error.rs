//! Implements Error type. It implements the `std::error::Error` for errors returned by ODBC and
//! allows for an error handling which his idomatic to rust

use std::fmt::{Display, Formatter};
use std;

/// ODBC Diagnostic Record
#[derive(Debug)]
pub struct DiagRec;

/// An ODBC Error
#[derive(Debug)]
pub enum Error {
    SqlError(DiagRec),
    EnvAllocFailure,
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        Ok(())
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "ODBC Error"
    }

    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}