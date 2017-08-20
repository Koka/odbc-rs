//! Implements the ODBC Environment
mod list_data_sources;
pub use self::list_data_sources::{DataSourceInfo, DriverInfo};
use super::{ffi, into_result, safe, try_into_option, EnvAllocError, GetDiagRec, Handle, Result};
use std;

/// Environment state used to represent that no odbc version has been set.
pub type NoVersion = safe::NoVersion;
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

impl Environment<NoVersion> {
    /// Allocates a new ODBC Environment
    ///
    /// After creation the `Environment` is in the `NoVersion` state. To do something with it you
    /// need to set the ODBC Version using `set_odbc_version_3`.
    ///
    /// # Example
    /// ```
    /// # use odbc::*;
    /// let env = match Environment::new(){
    ///     // Successful creation of Environment
    ///     Ok(env) => env,
    ///     // Sadly, we do not know the reason for failure, because there is no `Environment` to
    ///     // to get the `DiagnosticRecord` from.
    ///     Err(EnvAllocError) => panic!("Could not create an ODBC Environment."),
    /// };
    /// ```
    pub fn new() -> std::result::Result<Environment<NoVersion>, EnvAllocError> {

        match safe::Environment::new() {
            safe::Success(safe) => Ok(Environment { safe }),
            safe::Info(safe) => {
                warn!("{}", safe.get_diag_rec(1).unwrap());
                Ok(Environment { safe })
            }
            safe::Error(()) => Err(EnvAllocError),
        }
    }

    /// Tells the driver(s) that we will use features of up to ODBC version 3
    ///
    /// The first thing to do with an ODBC `Environment` is to set a version.
    ///
    /// # Example
    /// ```
    /// fn do_database_stuff() -> std::result::Result<(), Box<std::error::Error>> {
    ///     use odbc::*;
    ///     let env = Environment::new()?.set_odbc_version_3()?; // first thing to do
    ///     // ...
    ///     Ok(())
    /// }
    /// ```
    pub fn set_odbc_version_3(self) -> Result<Environment<Version3>> {
        let env = into_result(self.safe.declare_version_3())?;
        Ok(Environment { safe: env })
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
