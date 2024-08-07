//! Implements the ODBC Environment
mod list_data_sources;
pub use self::list_data_sources::{DataSourceInfo, DriverInfo};
use super::{
    ffi, into_result, safe, try_into_option, DiagnosticRecord, GetDiagRec, Handle, Result,
};
use std;

/// Environment state used to represent that environment has been set to odbc version 3
pub type Version3 = safe::Odbc3;

pub static mut OS_ENCODING: &encoding_rs::Encoding = encoding_rs::UTF_8;
pub static mut DB_ENCODING: &encoding_rs::Encoding = encoding_rs::UTF_8;

/// Handle to an ODBC Environment
///
/// Creating an instance of this type is the first thing you do then using ODBC. The environment
/// must outlive all connections created with it.
#[derive(Debug)]
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
    /// Creates an ODBC Environment and declares specification of `V` are used. You can use the
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
                warn!(
                    "{}",
                    v.get_diag_rec(1).unwrap_or_else(DiagnosticRecord::empty)
                );
                v
            }
            safe::Error(()) => return Err(None),
        };
        let safe = into_result(safe.declare_version())?;
        Ok(Environment { safe })
    }

    pub(crate) fn as_safe(&self) -> &safe::Environment<V> {
        &self.safe
    }
}

unsafe impl<V> safe::Handle for Environment<V> {
    const HANDLE_TYPE: ffi::HandleType = ffi::SQL_HANDLE_ENV;

    fn handle(&self) -> ffi::SQLHANDLE {
        self.safe.as_raw() as ffi::SQLHANDLE
    }
}

/// Creates an ODBC Environment and declares specification of version 3.0 are used
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
pub fn create_environment_v3(
) -> std::result::Result<Environment<Version3>, Option<DiagnosticRecord>> {
    Environment::new()
}

pub fn create_environment_v3_with_os_db_encoding(
    os_encoding: &str,
    db_encoding: &str,
) -> std::result::Result<Environment<Version3>, Option<DiagnosticRecord>> {
    unsafe {
        OS_ENCODING = encoding_rs::Encoding::for_label(os_encoding.as_bytes()).unwrap();
        DB_ENCODING = encoding_rs::Encoding::for_label(db_encoding.as_bytes()).unwrap();
    }
    Environment::new()
}
