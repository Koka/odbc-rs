use raw::{SQLFreeHandle, SQLHENV, SQL_HANDLE_ENV};

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