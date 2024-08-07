use odbc_safe::AutocommitMode;
use {
    ffi, Allocated, ColumnDescriptor, Handle, NoResult, Prepared, Raii, Result, ResultSetState,
    Return, Statement,
};

impl<'a, 'b, AC: AutocommitMode> Statement<'a, 'b, Allocated, NoResult, AC> {
    /// Prepares a statement for execution. Executing a prepared statement is faster than directly
    /// executing an unprepared statement, since it is already compiled into an Access Plan. This
    /// makes preparing statement a good idea if you want to repeatedly execute a query with a
    /// different set of parameters and care about performance.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use odbc::{*, safe};
    /// # fn doc() -> Result<()>{
    /// let env = create_environment_v3().map_err(|e| e.unwrap())?;
    /// let conn = env.connect("TestDataSource", "", "")?;
    /// let stmt = Statement::with_parent(&conn)?;
    /// let mut stmt = stmt.prepare("SELECT TITLE FROM MOVIES WHERE YEAR = ?")?;
    ///
    /// fn print_one_movie_from<'a> (year: u16, stmt: Statement<'a,'a, Prepared, NoResult, safe::AutocommitOn>) -> Result<Statement<'a, 'a, Prepared, NoResult, safe::AutocommitOn>>{
    ///    let stmt = stmt.bind_parameter(1, &year)?;
    ///    let stmt = if let Data(mut stmt) = stmt.execute()?{
    ///        if let Some(mut cursor) = stmt.fetch()?{
    ///            println!("{}", cursor.get_data::<String>(1)?.unwrap());
    ///        }
    ///        stmt.close_cursor()?
    ///    } else {
    ///       panic!("SELECT statement returned no result set");
    ///    };
    ///    stmt.reset_parameters()
    /// };
    ///
    /// for year in 1990..2010{
    ///     stmt = print_one_movie_from(year, stmt)?
    /// }
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn prepare(mut self, sql_text: &str) -> Result<Statement<'a, 'b, Prepared, NoResult, AC>> {
        self.raii.prepare(sql_text).into_result(&mut self)?;
        Ok(Statement::with_raii(self.raii))
    }

    /// Prepares a statement for execution. Executing a prepared statement is faster than directly
    /// executing an unprepared statement, since it is already compiled into an Access Plan. This
    /// makes preparing statement a good idea if you want to repeatedly execute a query with a
    /// different set of parameters and care about performance.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use odbc::*;
    /// # fn doc() -> Result<()>{
    /// let env = create_environment_v3().map_err(|e| e.unwrap())?;
    /// let conn = env.connect("TestDataSource", "", "")?;
    /// let stmt = Statement::with_parent(&conn)?;
    /// // need encode_rs crate
    /// // let mut stmt = stmt.prepare_bytes(&GB2312.encode("select '你好' as hello").0)?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn prepare_bytes(
        mut self,
        bytes: &[u8],
    ) -> Result<Statement<'a, 'b, Prepared, NoResult, AC>> {
        self.raii.prepare_byte(bytes).into_result(&mut self)?;
        Ok(Statement::with_raii(self.raii))
    }
}

impl<'a, 'b, AC: AutocommitMode> Statement<'a, 'b, Prepared, NoResult, AC> {
    /// The number of columns in a result set
    ///
    /// Can be called successfully only when the statement is in the prepared, executed, or
    /// positioned state. If the statement does not return columns the result will be 0.
    pub fn num_result_cols(&self) -> Result<i16> {
        self.raii.num_result_cols().into_result(self)
    }

    /// Returns description struct for result set column with a given index. Note: indexing is starting from 1.
    pub fn describe_col(&self, idx: u16) -> Result<ColumnDescriptor> {
        self.raii.describe_col(idx).into_result(self)
    }

    /// Executes a prepared statement.
    pub fn execute(mut self) -> Result<ResultSetState<'a, 'b, Prepared, AC>> {
        if self.raii.execute().into_result(&mut self)? {
            let num_cols = self.raii.num_result_cols().into_result(&self)?;
            if num_cols > 0 {
                Ok(ResultSetState::Data(Statement::with_raii(self.raii)))
            } else {
                Ok(ResultSetState::NoData(Statement::with_raii(self.raii)))
            }
        } else {
            Ok(ResultSetState::NoData(Statement::with_raii(self.raii)))
        }
    }
}

impl<'p> Raii<'p, ffi::Stmt> {
    fn prepare(&mut self, sql_text: &str) -> Return<()> {
        let bytes = unsafe { crate::environment::DB_ENCODING }
            .encode(sql_text)
            .0;
        match unsafe {
            ffi::SQLPrepare(
                self.handle(),
                bytes.as_ptr(),
                bytes.len() as ffi::SQLINTEGER,
            )
        } {
            ffi::SQL_SUCCESS => Return::Success(()),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(()),
            ffi::SQL_ERROR => Return::Error,
            r => panic!("SQLPrepare returned unexpected result: {:?}", r),
        }
    }

    fn prepare_byte(&mut self, bytes: &[u8]) -> Return<()> {
        match unsafe {
            ffi::SQLPrepare(
                self.handle(),
                bytes.as_ptr(),
                bytes.len() as ffi::SQLINTEGER,
            )
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
