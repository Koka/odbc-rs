use super::{Raii, ffi, Return, Handle};
use std::os::raw::{c_void, c_ulong};

impl Raii<ffi::Env> {
    pub fn set_odbc_version_3(&mut self) -> Return<()> {

        match unsafe {
            ffi::SQLSetEnvAttr(self.handle(),
                               ffi::SQL_ATTR_ODBC_VERSION,
                               ffi::SQL_OV_ODBC3 as c_ulong as *mut c_void,
                               0)
        } {
            ffi::SQL_SUCCESS => Return::Success(()),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(()),
            ffi::SQL_ERROR => Return::Error,
            _ => panic!("SQLSetEnvAttr returned an unexpected result"),
        }
    }
}
