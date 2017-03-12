
use super::{ffi, DataSource, Return, Result, Raii, Handle};
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
        let raii = Raii::with_parent(ds).into_result(ds)?;
        raii.tables().into_result(&raii)?;
        let stmt = Statement {
            raii: raii,
            parent: PhantomData,
        };
        Ok(stmt)
    }

    /// The number of columns in a result set
    ///
    /// Can be called successfully only when the statement is in the prepared, executed, or
    /// positioned state. If the statement does not return columns the result will be 0.
    pub fn num_result_cols(&self) -> Result<i16> {
        self.raii.num_result_cols().into_result(self)
    }
}

impl Raii<ffi::Stmt> {
    fn num_result_cols(&self) -> Return<i16> {
        let mut num_cols: ffi::SQLSMALLINT = 0;
        unsafe {
            match ffi::SQLNumResultCols(self.handle(), &mut num_cols as *mut ffi::SQLSMALLINT) {
                SQL_SUCCESS => Return::Success(num_cols),
                SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(num_cols),
                SQL_ERROR => Return::Error,
                SQL_STILL_EXECUTING => panic!("Multithreading currently impossible in safe code"),
                _ => unreachable!(),
            }
        }
    }

    fn tables(&self) -> Return<()> {
        let catalog_name = "";
        let schema_name = "";
        let table_name = "";
        let table_type = "TABLE";
        unsafe {
            match ffi::SQLTables(self.handle(),
                                 catalog_name.as_ptr(),
                                 catalog_name.as_bytes().len() as ffi::SQLSMALLINT,
                                 schema_name.as_ptr(),
                                 schema_name.as_bytes().len() as ffi::SQLSMALLINT,
                                 table_name.as_ptr(),
                                 table_name.as_bytes().len() as ffi::SQLSMALLINT,
                                 table_type.as_ptr(),
                                 table_type.as_bytes().len() as ffi::SQLSMALLINT) {
                SQL_SUCCESS => Return::Success(()),
                SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(()),
                SQL_ERROR => Return::Error,
                SQL_STILL_EXECUTING => panic!("Multithreading currently impossible in safe code"),
                _ => unreachable!(),
            }
        }
    }
}

