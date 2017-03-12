//! Implements the ODBC Environment
mod set_version;
mod list_data_sources;
pub use self::list_data_sources::{DataSourceInfo,DriverInfo};
use super::{Result, Return, ffi, GetDiagRec, Raii, Handle, EnvAllocError};
use std;
/// Handle to an ODBC Environment
///
/// Creating an instance of this type is the first thing you do then using ODBC. The environment
/// must outlive all connections created with it
pub struct Environment {
    raii: Raii<ffi::Env>,
}

impl Handle for Environment {
    type To = ffi::Env;
    unsafe fn handle(&self) -> ffi::SQLHENV {
        self.raii.handle()
    }
}

impl Environment {
    /// Allocates a new ODBC Environment
    ///
    /// Declares the Application's ODBC Version to be 3
    pub fn new() -> std::result::Result<Environment, EnvAllocError> {

        match unsafe { Raii::new() } {
            Return::Success(env) => Ok(Environment { raii: env }),
            Return::SuccessWithInfo(env) => {
                warn!("{}", env.get_diag_rec(1).unwrap());
                Ok(Environment { raii: env })
            }
            Return::Error => Err(EnvAllocError),
        }
    }

    /// Tells the driver(s) that we will use features of up to ODBC version 3
    pub fn set_odbc_version_3(&mut self) -> Result<()> {
        self.raii.set_odbc_version_3().into_result(self)
    }
}