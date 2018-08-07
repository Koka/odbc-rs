use std::error::Error;
use std::ffi::CString;
use safe::SqlStr;
use safe::sys::{SQL_NTS, SQL_NTSL, SQLCHAR};

pub struct SqlString(CString);

impl SqlString {
    pub fn from<S: Into<String>>(s: S) -> Result<Self, Box<Error>> {
        let cstring = CString::new(s.into())?;
        Ok(SqlString(cstring))
    }
}

unsafe impl SqlStr for SqlString {
    fn as_text_ptr(&self) -> *const u8 {
        self.0.as_ptr() as * const SQLCHAR
    }

    fn text_length(&self) -> i16 {
        SQL_NTS
    }

    fn text_length_int(&self) -> i32 {
        SQL_NTSL
    }
}
