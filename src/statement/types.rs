use raii::Raii;
use {ffi, Handle, Return};
use std::ptr::null_mut;
use std;

/// Implemented for fixed size type those representation is directly compatible with ODBC
pub unsafe trait FixedSizeType: Sized + Default {
    fn c_data_type() -> ffi::SqlCDataType;
}

/// Indicates that a type can be retrieved using `Cursor::get_data`
pub unsafe trait Output: Sized {
    fn get_data(stmt: &mut Raii<ffi::Stmt>,
                col_or_param_num: u16,
                buffer: &mut [u8])
                -> Return<Option<Self>>;
}

unsafe impl FixedSizeType for i16 {
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_SSHORT
    }
}

unsafe impl FixedSizeType for u16 {
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_USHORT
    }
}

unsafe impl FixedSizeType for i32 {
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_SLONG
    }
}

unsafe impl FixedSizeType for u32 {
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_ULONG
    }
}

unsafe impl FixedSizeType for f32 {
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_FLOAT
    }
}

unsafe impl FixedSizeType for f64 {
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_DOUBLE
    }
}

unsafe impl<T> Output for T
    where T: FixedSizeType
{
    fn get_data(stmt: &mut Raii<ffi::Stmt>,
                col_or_param_num: u16,
                _: &mut [u8])
                -> Return<Option<Self>> {
        stmt.get_data_fixed_size(col_or_param_num)
    }
}

unsafe impl Output for String {
    fn get_data(stmt: &mut Raii<ffi::Stmt>,
                col_or_param_num: u16,
                buffer: &mut [u8])
                -> Return<Option<Self>> {
        stmt.get_data_string(col_or_param_num, buffer)
    }
}

impl Raii<ffi::Stmt> {
    fn get_data_fixed_size<T>(&mut self, col_or_param_num: u16) -> Return<Option<T>>
        where T: FixedSizeType
    {
        let mut out = T::default();
        let mut indicator: ffi::SQLLEN = 0;
        unsafe {
            // Get buffer length...
            let result = ffi::SQLGetData(self.handle(),
                                         col_or_param_num,
                                         T::c_data_type(),
                                         &mut out as *mut T as ffi::SQLPOINTER,
                                         std::mem::size_of::<Self>() as ffi::SQLLEN,
                                         &mut indicator as *mut ffi::SQLLEN);
            match result {
                ffi::SQL_SUCCESS => {
                    if indicator == ffi::SQL_NULL_DATA {
                        Return::Success(None)
                    } else {
                        Return::Success(Some(out))
                    }
                }
                ffi::SQL_SUCCESS_WITH_INFO => {
                    if indicator == ffi::SQL_NULL_DATA {
                        Return::Success(None)
                    } else {
                        Return::Success(Some(out))
                    }
                }
                ffi::SQL_ERROR => Return::Error,
                ffi::SQL_NO_DATA => panic!("SQLGetData has already returned the colmun data"),
                r => panic!("unexpected return value from SQLGetData: {:?}", r),
            }
        }
    }

    fn get_data_string(&mut self,
                       col_or_param_num: u16,
                       buffer: &mut [u8])
                       -> Return<Option<String>> {
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
                r => panic!("unexpected return value from SQLGetData: {:?}", r),
            }
        }
    }
}

