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
//! # Internal Design
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

//#[macro_use]
//extern crate log;

#[macro_use]
extern crate lazy_static;

pub extern crate odbc_safe;
use odbc_safe as safe;

mod environment;
pub use environment::Env;

mod error;
pub use error::GenericError;

mod string;
pub use string::SqlString;

mod connection;
pub use connection::{Connection, ResultSet};
