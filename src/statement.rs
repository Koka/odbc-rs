
use {ffi, DataSource, Return, Result, Raii, Handle, Connected};
use ffi::SQLRETURN::*;
use std::marker::PhantomData;
use std::ptr::null_mut;
use std;

/// `Statement` state used to represent a freshly allocated connection
pub enum Allocated {}
/// `Statement` state used to represet an executed statement
pub enum Executed {}

/// RAII wrapper around ODBC statement
pub struct Statement<'a, S> {
    raii: Raii<ffi::Stmt>,
    // we use phantom data to tell the borrow checker that we need to keep the data source alive
    // for the lifetime of the statement
    parent: PhantomData<&'a DataSource<'a, Connected>>,
    state: PhantomData<S>,
}

/// Used to retrieve data from the fields of a query resul
pub struct Cursor<'a, 'b: 'a> {
    stmt: &'a mut Statement<'b, Executed>,
    buffer: [u8; 512],
}

impl<'a, S> Handle for Statement<'a, S> {
    type To = ffi::Stmt;
    unsafe fn handle(&self) -> ffi::SQLHSTMT {
        self.raii.handle()
    }
}

impl<'a, S> Statement<'a, S> {
    fn with_raii(raii: Raii<ffi::Stmt>) -> Self {
        Statement {
            raii: raii,
            parent: PhantomData,
            state: PhantomData,
        }
    }
}

impl<'a> Statement<'a, Allocated> {
    pub fn with_parent(ds: &'a DataSource<Connected>) -> Result<Self> {
        let raii = Raii::with_parent(ds).into_result(ds)?;
        Ok(Self::with_raii(raii))
    }

    pub fn tables<'b>(mut self) -> Result<Statement<'a, Executed>> {
        self.raii.tables().into_result(&self)?;
        Ok(Statement::with_raii(self.raii))
    }

    /// Executes a preparable statement, using the current values of the parameter marker variables
    /// if any parameters exist in the statement.
    ///
    /// `SQLExecDirect` is the fastest way to submit an SQL statement for one-time execution.
    pub fn exec_direct(mut self, statement_text: &str) -> Result<Statement<'a, Executed>> {
        assert!(self.raii.exec_direct(statement_text).into_result(&self)?);
        Ok(Statement::with_raii(self.raii))
    }
}

impl<'a> Statement<'a, Executed> {
    /// The number of columns in a result set
    ///
    /// Can be called successfully only when the statement is in the prepared, executed, or
    /// positioned state. If the statement does not return columns the result will be 0.
    pub fn num_result_cols(&self) -> Result<i16> {
        self.raii.num_result_cols().into_result(self)
    }

    /// Fetches the next rowset of data from the result set and returns data for all bound columns.
    ///
    /// # Return
    /// Returns false on the last row
    pub fn fetch<'b>(&'b mut self) -> Result<Option<Cursor<'b, 'a>>> {
        if self.raii.fetch().into_result(self)? {
            Ok(Some(Cursor {
                        stmt: self,
                        buffer: [0u8; 512],
                    }))
        } else {
            Ok(None)
        }
    }
}

impl<'a, 'b> Cursor<'a, 'b> {
    /// Retrieves data for a single column in the result set
    pub fn get_data(&mut self, col_or_param_num: u16) -> Result<Option<String>> {
        self.stmt.raii.get_data(col_or_param_num, &mut self.buffer).into_result(self.stmt)
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
                r => panic!("SQLNumResultCols returned unexpected result: {:?}", r),
            }
        }
    }

    fn exec_direct(&mut self, statement_text: &str) -> Return<bool> {
        let length = statement_text.len();
        if length > ffi::SQLINTEGER::max_value() as usize {
            panic!("Statement text too long");
        }
        match unsafe {
                  ffi::SQLExecDirect(self.handle(),
                                     statement_text.as_ptr(),
                                     length as ffi::SQLINTEGER)
              } {
            ffi::SQL_SUCCESS => Return::Success(true),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(true),
            ffi::SQL_ERROR => Return::Error,
            ffi::SQL_NEED_DATA => panic!("SQLExecDirec returned SQL_NEED_DATA"),
            ffi::SQL_NO_DATA => Return::Success(false),
            r => panic!("SQLExecDirect returned unexpected result: {:?}", r),
        }
    }

    /// Fetches the next rowset of data from the result set and returns data for all bound columns.
    fn fetch(&mut self) -> Return<bool> {
        match unsafe { ffi::SQLFetch(self.handle()) } {
            ffi::SQL_SUCCESS => Return::Success(true),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(true),
            ffi::SQL_ERROR => Return::Error,
            ffi::SQL_NO_DATA => Return::Success(false),
            r => panic!("SQLFetch returned unexpected result: {:?}", r),
        }
    }

    fn tables(&mut self) -> Return<()> {
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
                r => panic!("SQLTables returned: {:?}", r),
            }
        }
    }

    fn get_data(&mut self, col_or_param_num: u16, buffer: &mut [u8]) -> Return<Option<String>> {
        if buffer.len() == 0 {
            panic!("buffer length may not be null");
        }
        if buffer.len() > ffi::SQLLEN::max_value() as usize {
            panic!("buffer is larger than {} bytes", ffi::SQLLEN::max_value());
        }

        let mut indicator: ffi::SQLLEN = 0;
        unsafe {
            // Get buffer length...
            let result = ffi::SQLGetData(self.handle(),
                                         col_or_param_num,
                                         ffi::SQL_C_CHAR,
                                         buffer.as_mut_ptr() as ffi::SQLPOINTER,
                                         buffer.len() as ffi::SQLLEN,
                                         &mut indicator as *mut ffi::SQLLEN);
            match result {
                ffi::SQL_SUCCESS => {
                    if indicator == ffi::SQL_NULL_DATA {
                        Return::Success(None)
                    } else {
                        Return::Success(Some(std::str::from_utf8(&buffer[..(indicator as usize)])
                                                 .unwrap()
                                                 .to_owned()))
                    }
                }
                ffi::SQL_SUCCESS_WITH_INFO => {
                    if indicator == ffi::SQL_NO_TOTAL {
                        Return::SuccessWithInfo(None)
                    } else {
                        // Check if string has been truncated. String is also truncated if
                        // indicator is equal to BUF_LENGTH because of terminating nul
                        if indicator >= buffer.len() as ffi::SQLLEN {
                            let extra_space = (indicator as usize + 1) - (buffer.len() - 1);
                            let mut heap_buf = Vec::with_capacity((indicator as usize) + 1);
                            // Copy everything but the terminating zero into the new buffer
                            heap_buf.extend_from_slice(&buffer[..(buffer.len() - 1)]);
                            // increase length
                            heap_buf.extend(std::iter::repeat(0).take(extra_space));
                            // Get remainder of string
                            let ret = ffi::SQLGetData(self.handle(),
                                                      col_or_param_num,
                                                      ffi::SQL_C_CHAR,
                                                      heap_buf.as_mut_slice()[buffer.len() - 1..]
                                                          .as_mut_ptr() as
                                                      ffi::SQLPOINTER,
                                                      extra_space as ffi::SQLLEN,
                                                      null_mut());
                            heap_buf.pop();
                            let value = String::from_utf8(heap_buf).unwrap();
                            match ret {
                                ffi::SQL_SUCCESS => Return::Success(Some(value)),
                                ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(Some(value)),
                                ffi::SQL_ERROR => Return::Error,
                                r => panic!("SQLGetData returned {:?}", r),
                            }
                        } else {
                            // No truncation. Warning may be due to some other issue.
                            Return::SuccessWithInfo(Some(std::str::from_utf8(&buffer[..(indicator as
                                                                                 usize)])
                                                                 .unwrap()
                                                                 .to_owned()))
                        }
                    }
                }
                ffi::SQL_ERROR => Return::Error,
                ffi::SQL_NO_DATA => panic!("SQLGetData has already returned the colmun data"),
                _ => panic!("unexpected return value from SQLGetData"),
            }
        }
    }
}

