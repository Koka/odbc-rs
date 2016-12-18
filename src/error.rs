//! Implements Error type. It implements the `std::error::Error` for errors returned by ODBC and
//! allows for an error handling which his idomatic to rust

use super::raw;
use std::fmt::{Display, Formatter};
use std;

/// ODBC Diagnostic Record
#[derive(Debug)]
pub struct DiagRec {
    pub state: [raw::SQLCHAR; 6],
    pub native_error_pointer: raw::SQLINTEGER,
    pub message: String,
}

impl DiagRec {
    /// Reads first diagnostic record from handle
    ///
    /// This function will panic if no Diagnostic record is available. Its primary use is to create
    /// one after another ODBC function returned SQL_ERROR
    pub unsafe fn create(handle_type: raw::SQLSMALLINT, handle: raw::SQLHANDLE) -> DiagRec {
        let mut state: [raw::SQLCHAR; 6] = [0; 6];
        let mut native_error_pointer = 0;
        let mut message_length = 0;
        // first call determines message buffer length
        match raw::SQLGetDiagRec(handle_type,
                                 handle,
                                 1,
                                 std::ptr::null_mut(),
                                 std::ptr::null_mut(),
                                 std::ptr::null_mut(),
                                 0,
                                 &mut message_length as *mut raw::SQLSMALLINT) {
            raw::SQL_SUCCESS => (),
            raw::SQL_SUCCESS_WITH_INFO => (),
            raw::SQL_NO_DATA => panic!("No diagnostic record found"),
            _ => panic!("Error retrieving diagnostic record"),
        };
        let mut message_buffer: Vec<_> = (0..(message_length + 1)).map(|_| 0).collect();
        // second call fills message buffer
        match raw::SQLGetDiagRec(handle_type,
                                 handle,
                                 1,
                                 &mut state[0] as *mut raw::SQLCHAR,
                                 &mut native_error_pointer as *mut raw::SQLINTEGER,
                                 &mut message_buffer[0] as *mut raw::SQLCHAR,
                                 message_length + 1,
                                 std::ptr::null_mut()) {
            raw::SQL_SUCCESS => {
                message_buffer.pop(); //Drop terminating zero
                DiagRec {
                    state: state,
                    native_error_pointer: native_error_pointer,
                    message: String::from_utf8_unchecked(message_buffer),
                }
            }
            _ => panic!("Error retrieving diagnostic record"),
        }
    }
}

/// An ODBC Error
#[derive(Debug)]
pub enum Error {
    SqlError(DiagRec),
    EnvAllocFailure,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        use std::error::Error;
        write!(f, "{}", self.description())
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::SqlError(ref dr) => dr.message.as_str(),
            &Error::EnvAllocFailure => "Failed to allocate ODBC environment",
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}