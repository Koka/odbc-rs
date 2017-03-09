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
pub trait Handle{
    type To;
    /// Returns a valid handle to the odbc type.
    unsafe fn handle(&self) -> * mut Self::To;
}

fn as_out_buffer(buffer: &mut [u8]) -> *mut u8 {
    if buffer.len() == 0 {
        std::ptr::null_mut()
    } else {
        buffer.as_mut_ptr()
    }
}

fn as_buffer_length(n: usize) -> ffi::SQLSMALLINT {
    use std;
    if n > std::i16::MAX as usize {
        std::i16::MAX
    } else {
        n as i16
    }
}