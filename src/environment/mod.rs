//! Implements the ODBC Environment
mod set_version;
mod list_data_sources;
pub use self::list_data_sources::{DataSourceInfo, DriverInfo};
use super::{Result, Return, ffi, GetDiagRec, Raii, Handle, EnvAllocError};
use std::marker::PhantomData;
use std;

/// Environment state used to represent that no odbc version has been set.
pub enum NoVersion {}
/// Environment state used to represent that environment has been set to odbc version 3
pub enum Version3 {}

/// Handle to an ODBC Environment
///
/// Creating an instance of this type is the first thing you do then using ODBC. The environment
/// must outlive all connections created with it.
pub struct Environment<V> {
    raii: Raii<ffi::Env>,
    state: PhantomData<V>,
}

impl<V> Handle for Environment<V> {
    type To = ffi::Env;
    unsafe fn handle(&self) -> ffi::SQLHENV {
        self.raii.handle()
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

        match unsafe { Raii::new() } {
            Return::Success(env) => {
                Ok(Environment {
                    raii: env,
                    state: PhantomData,
                })
            }
            Return::SuccessWithInfo(env) => {
                warn!("{}", env.get_diag_rec(1).unwrap());
                Ok(Environment {
                    raii: env,
                    state: PhantomData,
                })
            }
            Return::Error => Err(EnvAllocError),
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
    pub fn set_odbc_version_3(mut self) -> Result<Environment<Version3>> {
        self.raii.set_odbc_version_3().into_result(&self)?;
        Ok(Environment {
            raii: self.raii,
            state: PhantomData,
        })
    }
}
