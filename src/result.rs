//! Result types to enabling handling of ODBC Errors
use super::{DiagnosticRecord, GetDiagRec};
use safe;
use std;

/// Result type returned by most functions in this crate
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
                let mut i = 1;
                while let Some(diag) = odbc_object.get_diag_rec(i) {
                    warn!("{}", diag);
                    i += 1;
                }
                Ok(value)
            }
            Return::Error => {
                // Return the first record
                let diag = odbc_object
                    .get_diag_rec(1)
                    .unwrap_or_else(DiagnosticRecord::empty);
                error!("{}", diag);
                let mut i = 2;
                // log the rest
                while let Some(diag) = odbc_object.get_diag_rec(i) {
                    error!("{}", diag);
                    i += 1;
                }
                Err(diag)
            }
        }
    }
}

// temporary glue code to odbc-safe
pub fn try_into_option<T, E, D>(ret: safe::ReturnOption<T, E>, handle: &D) -> Result<Option<T>>
where
    D: GetDiagRec,
{
    match ret {
        safe::ReturnOption::Success(value) => Ok(Some(value)),
        safe::ReturnOption::Info(value) => {
            let mut i = 1;
            while let Some(diag) = handle.get_diag_rec(i) {
                warn!("{}", diag);
                i += 1;
            }
            Ok(Some(value))
        }
        safe::ReturnOption::NoData(_) => Ok(None),
        safe::ReturnOption::Error(_) => {
            // Return the first record
            let diag = handle
                .get_diag_rec(1)
                .unwrap_or_else(DiagnosticRecord::empty);
            error!("{}", diag);
            let mut i = 2;
            // log the rest
            while let Some(diag) = handle.get_diag_rec(i) {
                error!("{}", diag);
                i += 1;
            }
            Err(diag)
        }
    }
}

// temporary glue code to odbc-safe
pub fn into_result<T, E>(ret: safe::Return<T, E>) -> Result<T>
where
    T: GetDiagRec,
    E: GetDiagRec,
{
    match ret {
        safe::Return::Success(value) => Ok(value),
        safe::Return::Info(value) => {
            let mut i = 1;
            while let Some(diag) = value.get_diag_rec(i) {
                warn!("{}", diag);
                i += 1;
            }
            Ok(value)
        }
        safe::Return::Error(value) => {
            // Return the first record
            let diag = value
                .get_diag_rec(1)
                .unwrap_or_else(DiagnosticRecord::empty);
            error!("{}", diag);
            let mut i = 2;
            // log the rest
            while let Some(diag) = value.get_diag_rec(i) {
                error!("{}", diag);
                i += 1;
            }
            Err(diag)
        }
    }
}

// temporary glue code to odbc-safe
pub fn into_result_with<D, T, E>(diag: &D, ret: safe::Return<T, E>) -> Result<T>
where
    D: GetDiagRec,
{
    match ret {
        safe::Return::Success(value) => Ok(value),
        safe::Return::Info(value) => {
            let mut i = 1;
            while let Some(diag_rec) = diag.get_diag_rec(i) {
                warn!("{}", diag_rec);
                i += 1;
            }
            Ok(value)
        }
        safe::Return::Error(_) => {
            // Return the first record
            let diag_rec = diag.get_diag_rec(1).unwrap_or_else(DiagnosticRecord::empty);
            error!("{}", diag_rec);
            let mut i = 2;
            // log the rest
            while let Some(diag_rec) = diag.get_diag_rec(i) {
                error!("{}", diag_rec);
                i += 1;
            }
            Err(diag_rec)
        }
    }
}
