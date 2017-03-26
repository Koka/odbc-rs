use {ffi, Return, Result, Raii, Handle, Statement};

/// Allows types to be used with `Statement::bind_parameter`
pub unsafe trait InputParameter {
    fn c_data_type(&self) -> ffi::SqlCDataType;
    fn column_size(&self) -> ffi::SQLULEN;
    fn decimal_digits(&self) -> ffi::SQLSMALLINT;
    fn value_ptr(&self) -> ffi::SQLPOINTER;
}

impl<'a, S> Statement<'a, S> {
    pub fn bind_parameter(&mut self, parameter_index: u16, value: u32) -> Result<()> {
        self.raii.bind_input_parameter(parameter_index, value).into_result(self)
    }
}

impl Raii<ffi::Stmt> {
    fn bind_input_parameter(&mut self, parameter_index: u16, value: u32) -> Return<()> {

        use std::ptr::null_mut;
        let indicator = null_mut();
        match unsafe {
                  ffi::SQLBindParameter(
                self.handle(),
                parameter_index,
                ffi::SQL_PARAM_INPUT,
                ffi::SQL_C_ULONG,
                ffi::SQL_UNKNOWN_TYPE,
                4, // column size
                0, // decimal digits
                &value as *const u32 as ffi::SQLPOINTER, // parameter value ptr
                0, // buffer length
                indicator // str len or ind ptr
            )
              } {
            ffi::SQL_SUCCESS => Return::Success(()),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(()),
            ffi::SQL_ERROR => Return::Error,
            r => panic!("Unexpected return from SQLBindParameter: {:?}", r),
        }
    }
}

