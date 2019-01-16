use {ffi, Return, Result, Raii, Handle, Statement};
use super::types::OdbcType;

impl<'a, 'b, S, R> Statement<'a, 'b, S, R> {
    /// Binds a parameter to a parameter marker in an SQL statement.
    ///
    /// # Result
    /// This method will destroy the statement and create a new one which may not outlive the bound
    /// parameter. This is to ensure that the statement will not derefernce an invalid pointer
    /// during execution.
    ///
    /// # Arguments
    /// * `parameter_index` - Index of the marker to bind to the parameter. Starting at `1`
    /// * `value` - Reference to bind to the marker
    ///
    /// # Example
    /// ```
    /// # use odbc::*;
    /// # fn do_odbc_stuff() -> std::result::Result<(), Box<std::error::Error>> {
    /// let env = create_environment_v3().map_err(|e| e.unwrap())?;
    /// let conn = env.connect("TestDataSource", "", "")?;
    /// let stmt = Statement::with_parent(&conn)?;
    /// let param = 1968;
    /// let stmt = stmt.bind_parameter(1, &param)?;
    /// let sql_text = "SELECT TITLE FROM MOVIES WHERE YEAR = ?";
    /// if let Data(mut stmt) = stmt.exec_direct(sql_text)? {
    ///     // ...
    /// }
    /// #   Ok(())
    /// # }
    /// ```
    pub fn bind_parameter<'c, T>(
        mut self,
        parameter_index: u16,
        value: &'c T,
    ) -> Result<Statement<'a, 'c, S, R>>
    where
        T: OdbcType<'c>,
        T: ?Sized,
        'b: 'c,
    {
        let ind = if value.value_ptr() == 0 as *const Self as ffi::SQLPOINTER {
            ffi::SQL_NULL_DATA
        } else {
            value.column_size() as ffi::SQLLEN
        };

        let ind_ptr = self.param_ind_buffers.alloc(parameter_index as usize, ind);

        self.raii
            .bind_input_parameter(parameter_index, value, ind_ptr)
            .into_result(&self)?;
        Ok(self)
    }

    /// Releasing all parameter buffers set by `bind_parameter`. This method consumes the statement
    /// and returns a new one those lifetime is no longer limited by the buffers bound.
    pub fn reset_parameters(mut self) -> Result<Statement<'a, 'a, S, R>> {
        self.param_ind_buffers.clear();
        self.raii.reset_parameters().into_result(&mut self)?;
        Ok(Statement::with_raii(self.raii))
    }
}

impl Raii<ffi::Stmt> {
    fn bind_input_parameter<'c, T>(&mut self, parameter_index: u16, value: &'c T, str_len_or_ind_ptr: *mut ffi::SQLLEN) -> Return<()>
    where
        T: OdbcType<'c>,
        T: ?Sized,
    {
        match unsafe {
            ffi::SQLBindParameter(
                self.handle(),
                parameter_index,
                ffi::SQL_PARAM_INPUT,
                T::c_data_type(),
                T::sql_data_type(),
                value.column_size(),
                value.decimal_digits(),
                value.value_ptr(),
                0, // buffer length
                str_len_or_ind_ptr, // Note that this ptr has to be valid until statement is executed
            )
        } {
            ffi::SQL_SUCCESS => Return::Success(()),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(()),
            ffi::SQL_ERROR => Return::Error,
            r => panic!("Unexpected return from SQLBindParameter: {:?}", r),
        }
    }

    fn reset_parameters(&mut self) -> Return<()> {
        match unsafe { ffi::SQLFreeStmt(self.handle(), ffi::SQL_RESET_PARAMS) } {
            ffi::SQL_SUCCESS => Return::Success(()),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(()),
            ffi::SQL_ERROR => Return::Error,
            r => panic!("SQLFreeStmt returned unexpected result: {:?}", r),
        }
    }
}
