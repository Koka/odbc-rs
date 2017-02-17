use raw::{SQLAllocHandle, SQLFreeHandle, SQLSetEnvAttr, SQLHENV, SQL_HANDLE_ENV,
          SQL_ATTR_ODBC_VERSION, SQL_OV_ODBC3};
use raw::SQLRETURN::*;
use std::ptr::null_mut;
use std::os::raw::c_void;

/// Safe wrapper around ODBC Environment handle
pub struct Environment {
    pub handle: SQLHENV,
}

/// Returned if an Environment is allocated
pub enum EnvAllocResult {
    /// Creation of the environment is a success
    Success(Environment),
    /// Successfully created environment with warnings
    SuccessWithInfo(Environment),
    /// Allocation failed
    Error,
}

/// Returned if an Environment is allocated
pub enum SetEnvAttrResult {
    Success,
    SuccessWithInfo,
    Error,
}

impl Environment {
    pub fn new() -> EnvAllocResult {

        use self::EnvAllocResult::*;

        let mut env = null_mut();
        match unsafe { SQLAllocHandle(SQL_HANDLE_ENV, null_mut(), &mut env) } {
            SQL_SUCCESS => Success(Environment { handle: env }),
            SQL_SUCCESS_WITH_INFO => SuccessWithInfo(Environment { handle: env }),
            SQL_ERROR => Error,
            _ => panic!("SQLAllocHandle returned an unexpected result"),
        }
    }

    pub fn set_odbc_version(&mut self) -> SetEnvAttrResult {
        unsafe {
            use self::SetEnvAttrResult::*;

            match SQLSetEnvAttr(self.handle,
                                SQL_ATTR_ODBC_VERSION,
                                SQL_OV_ODBC3 as *mut c_void,
                                0) {
                SQL_SUCCESS => Success,
                SQL_SUCCESS_WITH_INFO => SuccessWithInfo,
                SQL_ERROR => Error,
                _ => panic!("SQLSetEnvAttr returned an unexpected result"),
            }
        }
    }
}

impl Drop for Environment {
    fn drop(&mut self) {
        unsafe {
            SQLFreeHandle(SQL_HANDLE_ENV, self.handle);
        }
    }
}