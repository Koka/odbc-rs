//! # ODBC
//! Open Database Connectivity or short ODBC is a low level high performance interface
//! introduced by Microsoft to access relational data stores. This crate wraps the raw C interface
//! and is intended to be usable in safe and idiomatic Rust.
//!
//! [What is ODBC?](https://docs.microsoft.com/en-us/sql/odbc/reference/what-is-odbc)
//!
//! [ODBC Programmer's Reference]
//! (https://docs.microsoft.com/en-us/sql/odbc/reference/odbc-programmer-s-reference)
//!
//! # Internal Desgin
//!
//! While designed as a relatively low level wrapper around ODBC this crate tries to prevent many
//! errors at compile time. The borrow checker and the RAII (Resource Acquisition Is
//! Initialization) idiom should prevent any occurence of `SQL_INVALID_HANDLE` in safe code.
//!
//! Using the type system and the borrow checker this crate ensures that each method call happens
//! in a valid state and state transitions are modeled in the type system. This should eliminate
//! the possibility of function sequence errors in safe code.
//!
//! [Basic ODBC Application Steps]
//! (https://docs.microsoft.com/en-us/sql/odbc/reference/develop-app/basic-odbc-application-steps)
//!
//! [ODBC State Transitions]
//! (https://docs.microsoft.com/en-us/sql/odbc/reference/appendixes/appendix-b-odbc-state-transition-tables)

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
use result::Return;
use std::ptr::null_mut;
pub use diagnostics::{DiagnosticRecord, GetDiagRec};
pub use result::{Result, EnvAllocError};
pub use environment::*;
pub use data_source::{DataSource, Connected, Disconnected};
pub use statement::*;

/// Reflects the ability of a type to expose a valid handle
pub trait Handle {
    type To;
    /// Returns a valid handle to the odbc type.
    unsafe fn handle(&self) -> *mut Self::To;
}

fn as_out_buffer(buffer: &mut [u8]) -> *mut u8 {
    if buffer.len() == 0 {
        null_mut()
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
