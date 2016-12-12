use super::{Error, Result, raw};
use std;

/// Handle to an ODBC Environment
///
/// Creating an instance of this type is the first thing you do then using ODBC. The environment
/// must outlive all connections created with it
pub struct Environment {
    handle: raw::SQLHENV,
}

impl Environment {
    /// Allocates a new ODBC Environment
    ///
    /// Declares the Application's ODBC Version to be 3
    pub fn new() -> Result<Environment> {
        unsafe {
            let mut env = std::ptr::null_mut();
            let mut result =
                match raw::SQLAllocHandle(raw::SQL_HANDLE_ENV, std::ptr::null_mut(), &mut env) {
                    raw::SQL_SUCCESS => Environment { handle: env },
                    raw::SQL_SUCCESS_WITH_INFO => Environment { handle: env },
                    // Driver Manager failed to allocate environment
                    raw::SQL_ERROR => return Err(Error {}),
                    _ => unreachable!(),
                };
            // no leak if we return an error here, env handle is already wrapped and would be
            // dropped
            result.set_attribute(raw::SQL_ATTR_ODBC_VERSION,
                               raw::SQL_OV_ODBC3 as *mut std::os::raw::c_void,
                               0)?;

            Ok(result)
        }
    }

    /// Allows access to the raw ODBC handle
    pub unsafe fn raw(&mut self) -> raw::SQLHENV {
        self.handle
    }

    /// Allows setting attributes to Environment
    pub unsafe fn set_attribute(&mut self,
                                attribute: raw::SQLINTEGER,
                                value: raw::SQLPOINTER,
                                length: raw::SQLINTEGER)
                                -> Result<()> {
        match raw::SQLSetEnvAttr(self.handle, attribute, value, length) {
            raw::SQL_SUCCESS => Ok(()),
            raw::SQL_SUCCESS_WITH_INFO => Ok(()),
            _ => Err(Error {}),
        }
    }
}

impl Drop for Environment {
    fn drop(&mut self) {
        unsafe {
            raw::SQLFreeEnv(self.handle);
        }
    }
}