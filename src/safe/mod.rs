use raw::{SQLFreeHandle, SQLHENV, SQL_HANDLE_ENV, SQLAllocHandle};
use raw::SQLRETURN::*;
use std::ptr::null_mut;

/// Safe wrapper around ODBC Environment handle
pub struct Environment {
    pub handle: SQLHENV,
}

/// Returned if an Environment is allocated
pub enum EnvAllocResult {
    Success(Environment),
    SuccessWithInfo(Environment),
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
}

impl Drop for Environment {
    fn drop(&mut self) {
        unsafe {
            SQLFreeHandle(SQL_HANDLE_ENV, self.handle);
        }
    }
}