//! Result types to enabling handling of ODBC Errors
use super::{DiagnosticRecord, GetDiagRec};
use std::fmt::{Display, Formatter};
use std;

pub type Result<T> = std::result::Result<T, DiagnosticRecord>;

#[must_use]
pub enum Return<T> {
    Success(T),
    SuccessWithInfo(T),
    Error,
}

impl<T> Return<T> {
    pub fn into_result<O: GetDiagRec>(self, odbc_object: &O) -> Result<T> {
        match self {
            Return::Success(value) => Ok(value),
            Return::SuccessWithInfo(value) => {
                warn!("{}", odbc_object.get_diag_rec(1).unwrap());
                Ok(value)
            }
            Return::Error => {
                let diag = odbc_object.get_diag_rec(1).unwrap();
                error!("{}", diag);
                Err(diag)
            }
        }
    }
}

/// Environment allocation error
///
/// Allocating an environment is the one operation in ODBC which does not yiel a diagnostic record
/// in case of an error. There is simply no Environment to ask for a diagnostic record
#[derive(Debug)]
pub struct EnvAllocError;

impl Display for EnvAllocError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        use std::error::Error;
        write!(f, "{}", self.description())
    }
}

impl std::error::Error for EnvAllocError {
    fn description(&self) -> &str {
        "Failure to allocate ODBC environment"
    }

    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}

