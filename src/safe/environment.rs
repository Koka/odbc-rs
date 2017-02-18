use super::Handle;
use raw::{SQLAllocHandle, SQLFreeHandle, SQLSetEnvAttr, SQLRETURN, SQLHENV, SQLHANDLE,
          SQLSMALLINT, SQL_HANDLE_ENV, SQL_ATTR_ODBC_VERSION, SQL_OV_ODBC3};
use std::ptr::null_mut;
use std::os::raw::c_void;

/// Safe wrapper around ODBC Environment handle
pub struct Environment {
    pub handle: SQLHENV,
}

impl Drop for Environment {
    fn drop(&mut self) {
        unsafe {
            SQLFreeHandle(SQL_HANDLE_ENV, self.handle);
        }
    }
}

unsafe impl Handle for Environment {
    fn handle(&self) -> SQLHANDLE {
        self.handle
    }

    fn handle_type() -> SQLSMALLINT {
        SQL_HANDLE_ENV
    }
}

impl Environment {
    pub fn new() -> EnvAllocResult {

        use self::EnvAllocResult::*;

        let mut env = null_mut();
        match unsafe { SQLAllocHandle(SQL_HANDLE_ENV, null_mut(), &mut env) } {
            SQLRETURN::SQL_SUCCESS => Success(Environment { handle: env }),
            SQLRETURN::SQL_SUCCESS_WITH_INFO => SuccessWithInfo(Environment { handle: env }),
            SQLRETURN::SQL_ERROR => Error,
            _ => panic!("SQLAllocHandle returned an unexpected result"),
        }
    }

    pub fn set_odbc_version_3(&mut self) -> SetEnvAttrResult {

        use self::SetEnvAttrResult::*;

        match unsafe {
            SQLSetEnvAttr(self.handle,
                          SQL_ATTR_ODBC_VERSION,
                          SQL_OV_ODBC3 as *mut c_void,
                          0)
        } {
            SQLRETURN::SQL_SUCCESS => Success,
            SQLRETURN::SQL_SUCCESS_WITH_INFO => SuccessWithInfo,
            SQLRETURN::SQL_ERROR => Error,
            _ => panic!("SQLSetEnvAttr returned an unexpected result"),
        }
    }
}

/// Returned if an Environment is allocated
#[must_use]
pub enum EnvAllocResult {
    /// Creation of the environment is a success
    Success(Environment),
    /// Successfully created environment with warnings
    SuccessWithInfo(Environment),
    /// Allocation failed
    Error,
}

/// Returned if an Environment is allocated
#[must_use]
pub enum SetEnvAttrResult {
    Success,
    SuccessWithInfo,
    Error,
}