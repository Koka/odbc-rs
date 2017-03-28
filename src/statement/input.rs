use {ffi, Return, Result, Raii, Handle, Statement};
use super::types::FixedSizedType;

/// Allows types to be used with `Statement::bind_parameter`
pub unsafe trait InputParameter {
    fn c_data_type(&self) -> ffi::SqlCDataType;
    fn column_size(&self) -> ffi::SQLULEN;
    fn decimal_digits(&self) -> ffi::SQLSMALLINT;
    fn value_ptr(&self) -> ffi::SQLPOINTER;
    fn indicator(&self) -> ffi::SQLLEN;
}

impl<'a, 'b, S> Statement<'a, 'b, S> {
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
    /// let env = Environment::new()?.set_odbc_version_3()?;
    /// let conn = DataSource::with_parent(&env)?.connect("TestDataSource", "", "")?;
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
    pub fn bind_parameter<'c, T>(mut self,
                                 parameter_index: u16,
                                 value: &'c T)
                                 -> Result<Statement<'a, 'c, S>>
        where T: InputParameter,
              T: ?Sized,
              'b: 'c
    {
        self.raii.bind_input_parameter(parameter_index, value).into_result(&self)?;
        Ok(self)
    }
}

impl Raii<ffi::Stmt> {
    fn bind_input_parameter<T>(&mut self, parameter_index: u16, value: &T) -> Return<()>
        where T: InputParameter,
              T: ?Sized
    {
        match unsafe {
                  ffi::SQLBindParameter(
                self.handle(),
                parameter_index,
                ffi::SQL_PARAM_INPUT,
                value.c_data_type(),
                ffi::SQL_UNKNOWN_TYPE,
                value.column_size(),
                value.decimal_digits(),
                value.value_ptr(),
                0, // buffer length
                &value.indicator() as * const ffi::SQLLEN as * mut ffi::SQLLEN// str len or ind ptr
            )
              } {
            ffi::SQL_SUCCESS => Return::Success(()),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(()),
            ffi::SQL_ERROR => Return::Error,
            r => panic!("Unexpected return from SQLBindParameter: {:?}", r),
        }
    }
}

unsafe impl InputParameter for str {
    fn c_data_type(&self) -> ffi::SqlCDataType {
        ffi::SQL_C_CHAR
    }

    fn column_size(&self) -> ffi::SQLULEN {
        self.as_bytes().len() as ffi::SQLULEN
    }

    fn decimal_digits(&self) -> ffi::SQLSMALLINT {
        0
    }

    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self.as_bytes().as_ptr() as ffi::SQLPOINTER
    }

    fn indicator(&self) -> ffi::SQLLEN {
        self.as_bytes().len() as ffi::SQLLEN
    }
}

unsafe impl InputParameter for String {
    fn c_data_type(&self) -> ffi::SqlCDataType {
        ffi::SQL_C_CHAR
    }

    fn column_size(&self) -> ffi::SQLULEN {
        self.as_bytes().len() as ffi::SQLULEN
    }

    fn decimal_digits(&self) -> ffi::SQLSMALLINT {
        0
    }

    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self.as_bytes().as_ptr() as ffi::SQLPOINTER
    }

    fn indicator(&self) -> ffi::SQLLEN {
        self.as_bytes().len() as ffi::SQLLEN
    }
}

unsafe impl<T> InputParameter for T
    where T: FixedSizedType
{
    fn c_data_type(&self) -> ffi::SqlCDataType {
        T::c_data_type()
    }

    fn column_size(&self) -> ffi::SQLULEN {
        use std::mem::size_of;
        size_of::<Self>() as ffi::SQLULEN
    }

    fn decimal_digits(&self) -> ffi::SQLSMALLINT {
        0
    }

    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self as *const Self as ffi::SQLPOINTER
    }

    fn indicator(&self) -> ffi::SQLLEN {
        0
    }
}

