#[macro_use]
extern crate log;

pub mod ffi;
mod odbc_object;
mod raii;
mod diagnostics;
mod result;
mod environment;
mod data_source;
mod statement;
use odbc_object::OdbcObject;
use raii::Raii;
pub use diagnostics::{DiagnosticRecord, GetDiagRec};
pub use result::*;
pub use environment::*;
pub use data_source::DataSource;
pub use statement::*;

/// Reflects the ability of a type to expose a valid handle
pub trait Handle {
    type To;
    /// Returns a valid handle to the odbc type.
    unsafe fn handle(&self) -> *mut Self::To;
}
