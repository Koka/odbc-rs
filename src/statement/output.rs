use raii::Raii;
use {ffi, Handle, Return};
use super::types::OdbcType;
use std;

/// Indicates that a type can be retrieved using `Cursor::get_data`
pub unsafe trait Output<'a>: Sized {
    fn get_data(
        stmt: &mut Raii<ffi::Stmt>,
        col_or_param_num: u16,
        buffer: &'a mut Vec<u8>,
    ) -> Return<Option<Self>>;
}

unsafe impl<'a, T> Output<'a> for T
where
    T: OdbcType<'a>,
{
    fn get_data(
        stmt: &mut Raii<ffi::Stmt>,
        col_or_param_num: u16,
        buffer: &'a mut Vec<u8>,
    ) -> Return<Option<Self>> {
        stmt.get_data(col_or_param_num, buffer)
    }
}

impl Raii<ffi::Stmt> {
    fn get_data<'a, T>(
        &mut self,
        col_or_param_num: u16,
        buffer: &'a mut Vec<u8>,
    ) -> Return<Option<T>>
    where
        T: OdbcType<'a>,
    {
        if buffer.len() == 0 {
            panic!("buffer length may not be zero");
        }
        if buffer.len() > ffi::SQLLEN::max_value() as usize {
            panic!("buffer is larger than {} bytes", ffi::SQLLEN::max_value());
        }
        let mut indicator: ffi::SQLLEN = 0;
        unsafe {
            // Get buffer length...
            let result = ffi::SQLGetData(
                self.handle(),
                col_or_param_num,
                T::c_data_type(),
                buffer.as_mut_ptr() as ffi::SQLPOINTER,
                buffer.len() as ffi::SQLLEN,
                &mut indicator as *mut ffi::SQLLEN,
            );
            match result {
                ffi::SQL_SUCCESS => {
                    if indicator == ffi::SQL_NULL_DATA {
                        Return::Success(None)
                    } else {
                        let slice = &buffer[..(indicator as usize)];
                        Return::Success(Some(T::convert(slice)))
                    }
                }
                ffi::SQL_SUCCESS_WITH_INFO => {
                    if indicator == ffi::SQL_NO_TOTAL {
                        Return::SuccessWithInfo(None)
                    } else {
                        // Check if string has been truncated.
                        // String is also truncated if indicator is equal to BUF_LENGTH because of terminating nul
                        if indicator >= buffer.len() as ffi::SQLLEN {
                            let extra_space = (indicator as usize + 1) - (buffer.len() - 1);

                            let old_len = buffer.len();

                            let mut tmp = Vec::with_capacity((indicator as usize) + 1);
                            // Copy everything but the terminating zero into the new buffer
                            tmp.extend_from_slice(&buffer[..(old_len - 1)]);
                            // increase length
                            tmp.extend(std::iter::repeat(0).take(extra_space));

                            buffer.clear();
                            buffer.extend_from_slice(&tmp);

                            // Get remainder of string
                            let ret = ffi::SQLGetData(
                                self.handle(),
                                col_or_param_num,
                                T::c_data_type(),
                                buffer.as_mut_slice()[old_len - 1..].as_mut_ptr() as
                                    ffi::SQLPOINTER,
                                extra_space as ffi::SQLLEN,
                                std::ptr::null_mut(),
                            );
                            buffer.pop();
                            let value = T::convert(buffer);
                            match ret {
                                ffi::SQL_SUCCESS => Return::Success(Some(value)),
                                ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(Some(value)),
                                ffi::SQL_ERROR => Return::Error,
                                r => panic!("SQLGetData returned {:?}", r),
                            }
                        } else {
                            let slice = &buffer[..(indicator as usize)];
                            // No truncation. Warning may be due to some other issue.
                            Return::SuccessWithInfo(Some(T::convert(slice)))
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
