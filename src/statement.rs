
use super::{ffi, DataSource, Error, Return, Result, Raii, GetDiagRec, Handle};
use super::ffi::SQLRETURN::*;
use std::marker::PhantomData;

/// RAII wrapper around ODBC statement
pub struct Statement<'a> {
    raii: Raii<ffi::Stmt>,
    // we use phantom data to tell the borrow checker that we need to keep the data source alive
    // for the lifetime of the statement
    parent: PhantomData<&'a DataSource<'a>>,
}

impl<'a> Handle for Statement<'a> {
    type To = ffi::Stmt;
    unsafe fn handle(&self) -> ffi::SQLHSTMT {
        self.raii.handle()
    }
}

impl<'a> Statement<'a> {
    pub fn with_tables<'b>(ds: &'b mut DataSource) -> Result<Statement<'b>> {
        let raii = match Raii::with_parent(ds) {
            Return::Success(s) => s,
            Return::SuccessWithInfo(s) => s,
            Return::Error => return Err(Error::SqlError(ds.get_diag_rec(1).unwrap())),
        };

        let stmt = Statement {
            raii: raii,
            parent: PhantomData,
        };

        let catalog_name = "";
        let schema_name = "";
        let table_name = "";
        let table_type = "TABLE";
        unsafe {
            match ffi::SQLTables(stmt.raii.handle(),
                                 catalog_name.as_ptr(),
                                 catalog_name.as_bytes().len() as ffi::SQLSMALLINT,
                                 schema_name.as_ptr(),
                                 schema_name.as_bytes().len() as ffi::SQLSMALLINT,
                                 table_name.as_ptr(),
                                 table_name.as_bytes().len() as ffi::SQLSMALLINT,
                                 table_type.as_ptr(),
                                 table_type.as_bytes().len() as ffi::SQLSMALLINT) {
                SQL_SUCCESS |
                SQL_SUCCESS_WITH_INFO => Ok(stmt),
                SQL_ERROR => Err(Error::SqlError(stmt.get_diag_rec(1).unwrap())),
                SQL_STILL_EXECUTING => panic!("Multithreading currently impossible in safe code"),
                _ => unreachable!(),
            }
        }
    }

    /// The number of columns in a result set
    ///
    /// Can be called successfully only when the statement is in the prepared, executed, or
    /// positioned state. If the statement does not return columns the result will be 0.
    pub fn num_result_cols(&self) -> Result<i16> {
        let mut num_cols: ffi::SQLSMALLINT = 0;
        unsafe {
            match ffi::SQLNumResultCols(self.raii.handle(),
                                        &mut num_cols as *mut ffi::SQLSMALLINT) {
                SQL_SUCCESS |
                SQL_SUCCESS_WITH_INFO => Ok(num_cols),
                SQL_ERROR => Err(Error::SqlError(self.get_diag_rec(1).unwrap())),
                SQL_STILL_EXECUTING => panic!("Multithreading currently impossible in safe code"),
                _ => unreachable!(),
            }
        }
    }
}

