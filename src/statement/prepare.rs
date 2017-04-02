use {ffi, Raii, Return, Handle, Statement, Result, Prepared, Allocated, NoResult, ResultSetState};

impl<'a, 'b> Statement<'a, 'b, Allocated, NoResult> {
    /// Prepares a statement for execution
    pub fn prepare(mut self, sql_text: &str) -> Result<Statement<'a, 'b, Prepared, NoResult>> {
        self.raii.prepare(sql_text).into_result(&mut self)?;
        Ok(Statement::with_raii(self.raii))
    }
}

impl<'a, 'b> Statement<'a, 'b, Prepared, NoResult> {
    pub fn execute(mut self) -> Result<ResultSetState<'a, 'b, Prepared>> {
        if self.raii.execute().into_result(&mut self)? {
            Ok(ResultSetState::Data(Statement::with_raii(self.raii)))
        } else {
            Ok(ResultSetState::NoData(Statement::with_raii(self.raii)))
        }
    }
}

impl Raii<ffi::Stmt> {
    fn prepare(&mut self, sql_text: &str) -> Return<()> {
        match unsafe {
                  ffi::SQLPrepare(self.handle(),
                                  sql_text.as_bytes().as_ptr(),
                                  sql_text.as_bytes().len() as ffi::SQLINTEGER)
              } {
            ffi::SQL_SUCCESS => Return::Success(()),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(()),
            ffi::SQL_ERROR => Return::Error,
            r => panic!("SQLPrepare returned unexpected result: {:?}", r),
        }
    }

    fn execute(&mut self) -> Return<bool> {
        match unsafe { ffi::SQLExecute(self.handle()) } {
            ffi::SQL_SUCCESS => Return::Success(true),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(true),
            ffi::SQL_ERROR => Return::Error,
            ffi::SQL_NO_DATA => Return::Success(false),
            r => panic!("SQLExecute returned unexpected result: {:?}", r),
        }
    }
}

