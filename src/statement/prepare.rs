use {ffi, Raii, Return, Handle, Statement, Result, Prepared, Allocated, NoResult, ResultSetState};

impl<'a, 'b> Statement<'a, 'b, Allocated, NoResult> {
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
    /// let env = Environment::new().unwrap().set_odbc_version_3()?;
    /// let conn = DataSource::with_parent(&env)?.connect("TestDataSource", "", "")?;
    /// let stmt = Statement::with_parent(&conn)?;
    /// let mut stmt = stmt.prepare("SELECT TITLE FROM MOVIES WHERE YEAR = ?").unwrap();
    ///
    /// fn print_one_movie_from<'a> (year: u16, stmt: Statement<'a,'a, Prepared, NoResult>) -> Result<Statement<'a, 'a, Prepared, NoResult>>{
    ///    let stmt = stmt.bind_parameter(1, &year)?;
    ///    let stmt = if let Data(mut stmt) = stmt.execute()?{
    ///        {
    ///            let mut cursor = stmt.fetch()?.unwrap();
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
    pub fn prepare(mut self, sql_text: &str) -> Result<Statement<'a, 'b, Prepared, NoResult>> {
        self.raii.prepare(sql_text).into_result(&mut self)?;
        Ok(Statement::with_raii(self.raii))
    }
}

impl<'a, 'b> Statement<'a, 'b, Prepared, NoResult> {
    /// Executes a prepared statement.
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
