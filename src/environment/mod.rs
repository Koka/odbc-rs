//! Implements the ODBC Environment
mod list_data_sources;
pub use self::list_data_sources::{DataSourceInfo, DriverInfo};
use super::{ffi, into_result, safe, try_into_option, DiagnosticRecord, GetDiagRec, Handle, Result};
use std;

/// Environment state used to represent that environment has been set to odbc version 3
pub type Version3 = safe::Odbc3;

/// Handle to an ODBC Environment
///
/// Creating an instance of this type is the first thing you do then using ODBC. The environment
/// must outlive all connections created with it.
pub struct Environment<V> {
    safe: safe::Environment<V>,
}

impl<V> Handle for Environment<V> {
    type To = ffi::Env;
    unsafe fn handle(&self) -> ffi::SQLHENV {
        self.safe.as_raw()
    }
}

impl<V: safe::Version> Environment<V> {
    /// Creates an ODBC Environment and declares specifaciton of `V` are used. You can use the
    /// shorthand `create_environment_v3()` instead.
    ///
    /// # Example
    /// ```
    /// use odbc::*;
    /// fn do_database_stuff() -> std::result::Result<(), Option<DiagnosticRecord>> {
    ///     let env : Environment<Version3> = Environment::new()?; // first thing to do
    ///     // ...
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Return
    ///
    /// While most functions in this crate return a `DiagnosticRecord` in the event of an Error the
    /// creation of an environment is special. Since `DiagnosticRecord`s are created using the
    /// environment, at least its allocation has to be successful to obtain one. If the allocation
    /// fails it is sadly not possible to receive further Diagnostics. Setting an unsupported version
    /// may however result in an ordinary `Some(DiagnosticRecord)`.
    /// ```
    pub fn new() -> std::result::Result<Environment<V>, Option<DiagnosticRecord>> {
        let safe = match safe::Environment::new() {
            safe::Success(v) => v,
            safe::Info(v) => {
                warn!("{}", v.get_diag_rec(1).unwrap());
                v
            }
            safe::Error(()) => return Err(None),
        };
        let safe = into_result(safe.declare_version())?;
        Ok(Environment { safe })
    }
}

unsafe impl<V> safe::Handle for Environment<V> {
    fn handle(&self) -> ffi::SQLHANDLE {
        self.safe.as_raw() as ffi::SQLHANDLE
    }

    fn handle_type() -> ffi::HandleType {
        ffi::SQL_HANDLE_ENV
    }
}

/// Creates an ODBC Environment and declares specifaciton of version 3.0 are used
///
/// # Example
/// ```
/// use odbc::*;
/// fn do_database_stuff() -> std::result::Result<(), Option<DiagnosticRecord>> {
///     let env = create_environment_v3()?; // first thing to do
///     // ...
///     Ok(())
/// }
/// ```
///
/// # Return
///
/// While most functions in this crate return a `DiagnosticRecord` in the event of an Error the
/// creation of an environment is special. Since `DiagnosticRecord`s are created using the
/// environment, at least its allocation has to be successful to obtain one. If the allocation
/// fails it is sadly not possible to receive further Diagnostics. Setting an unsupported version
/// may however result in an ordinary `Some(DiagnosticRecord)`.
pub fn create_environment_v3()
    -> std::result::Result<Environment<Version3>, Option<DiagnosticRecord>>
{
    Environment::new()
}
