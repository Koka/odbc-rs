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
extern crate odbc_safe;

pub mod ffi;

pub use diagnostics::{DiagnosticRecord, GetDiagRec};
pub use result::Result;
pub use environment::*;
pub use connection::Connection;
pub use statement::*;

use odbc_object::OdbcObject;
use raii::Raii;
use result::{Return, into_result, try_into_option};
use odbc_safe as safe;

mod odbc_object;
mod raii;
mod diagnostics;
mod result;
mod environment;
mod connection;
mod statement;


/// Reflects the ability of a type to expose a valid handle
pub trait Handle {
    type To;
    /// Returns a valid handle to the odbc type.
    unsafe fn handle(&self) -> *mut Self::To;
}
