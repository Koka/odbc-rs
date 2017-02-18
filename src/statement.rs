
use super::{raw, DataSource, Error, Result};
use safe::{Handle, GetDiagRec};
use super::raw::SQLRETURN::*;
use std::marker::PhantomData;
use std::ptr::null_mut;

/// RAII wrapper around ODBC statement
pub struct Statement<'a> {
    handle: raw::SQLHSTMT,
    // we use phantom data to tell the borrow checker that we need to keep the data source alive
    // for the lifetime of the statement
    parent: PhantomData<&'a DataSource<'a>>,
}

impl<'a> Drop for Statement<'a> {
    fn drop(&mut self) {
        unsafe {
            raw::SQLFreeHandle(raw::SQL_HANDLE_STMT, self.handle);
        }
    }
}

unsafe impl<'a> Handle for Statement<'a> {
    fn handle(&self) -> raw::SQLHANDLE {
        self.handle
    }

    fn handle_type() -> raw::SQLSMALLINT {
        raw::SQL_HANDLE_STMT
    }
}

impl<'a> Statement<'a> {
    pub fn with_tables<'b>(ds: &'b mut DataSource) -> Result<Statement<'b>> {
        let stmt = Statement::allocate(ds)?;
        let catalog_name = "";
        let schema_name = "";
        let table_name = "";
        let table_type = "TABLE";
        unsafe {
            match raw::SQLTables(stmt.handle,
                                 catalog_name.as_ptr(),
                                 catalog_name.as_bytes().len() as raw::SQLSMALLINT,
                                 schema_name.as_ptr(),
                                 schema_name.as_bytes().len() as raw::SQLSMALLINT,
                                 table_name.as_ptr(),
                                 table_name.as_bytes().len() as raw::SQLSMALLINT,
                                 table_type.as_ptr(),
                                 table_type.as_bytes().len() as raw::SQLSMALLINT) {
                SQL_SUCCESS |
                SQL_SUCCESS_WITH_INFO => Ok(stmt),
                SQL_ERROR => Err(Error::SqlError(stmt.get_diagnostic_record(1).unwrap())),
                SQL_STILL_EXECUTING => panic!("Multithreading currently impossible in safe code"),
                _ => unreachable!(),
            }
        }
    }

    fn allocate<'b>(parent: &'b mut DataSource) -> Result<Statement<'b>> {
        unsafe {
            let mut stmt = null_mut();
            match raw::SQLAllocHandle(raw::SQL_HANDLE_STMT, parent.raw(), &mut stmt) {
                SQL_SUCCESS |
                SQL_SUCCESS_WITH_INFO => {
                    Ok(Statement {
                        handle: stmt,
                        parent: PhantomData,
                    })
                }
                // Driver Manager failed to allocate statement
                SQL_ERROR => Err(Error::SqlError(parent.get_diagnostic_record(1).unwrap())),
                _ => unreachable!(),
            }
        }
    }

    /// The number of columns in a result set
    ///
    /// Can be called successfully only when the statement is in the prepared, executed, or
    /// positioned state. If the statement does not return columns the result will be 0.
    pub fn num_result_cols(&self) -> Result<i16> {
        let mut num_cols: raw::SQLSMALLINT = 0;
        unsafe {
            match raw::SQLNumResultCols(self.handle, &mut num_cols as *mut raw::SQLSMALLINT) {
                SQL_SUCCESS |
                SQL_SUCCESS_WITH_INFO => Ok(num_cols),
                SQL_ERROR => Err(Error::SqlError(self.get_diagnostic_record(1).unwrap())),
                SQL_STILL_EXECUTING => panic!("Multithreading currently impossible in safe code"),
                _ => unreachable!(),
            }
        }
    }

    /// Allows access to the raw ODBC handle
    pub unsafe fn raw(&mut self) -> raw::SQLHSTMT {
        self.handle
    }
}
