use super::{Handle, OdbcObject};
use ffi::{SQLGetDiagRec, SQLSMALLINT, SQLRETURN, SQLINTEGER, SQLHANDLE};
use std::ptr::null_mut;
use std::fmt;

/// ODBC Diagonstic record
#[derive(Debug)]
pub struct DiagnosticRecord {
    pub state: [u8; 6],
    pub native_error_pointer: i32,
    pub message: String,
}

impl fmt::Display for DiagnosticRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub trait GetDiagRec {
    /// Retrieves a diagnostic record
    ///
    /// `record_number` - Record numbers start at one. If you pass an number < 1 the function will
    /// panic. If no record is available for the number specified none is returned.
    fn get_diag_rec(&self, record_number: i16) -> Option<DiagnosticRecord>;
}

impl<H> GetDiagRec for H
    where H: Handle,
          H::To: OdbcObject
{
    fn get_diag_rec(&self, record_number: i16) -> Option<DiagnosticRecord> {
        // Call SQLGetDiagRec two times. First time to get the message text length, the second
        // to fill the result with diagnostic information
        let mut text_length: SQLSMALLINT = 0;

        match unsafe {
                  SQLGetDiagRec(H::To::handle_type(),
                                self.handle() as SQLHANDLE,
                                record_number,
                                null_mut(),
                                null_mut(),
                                null_mut(),
                                0,
                                &mut text_length as *mut SQLSMALLINT)
              } {
            SQLRETURN::SQL_SUCCESS |
            SQLRETURN::SQL_SUCCESS_WITH_INFO => (),
            SQLRETURN::SQL_ERROR => {
                if record_number > 0 {
                    panic!("SQLGetDiagRec returned an unexpected result")
                } else {
                    panic!("record number start at 1 has been {}", record_number)
                }
            }
            SQLRETURN::SQL_NO_DATA => return None,
            _ => panic!("SQLGetDiagRec returned an unexpected result"),
        }

        let mut message = vec![0; (text_length + 1) as usize];
        let mut result = DiagnosticRecord {
            state: [0; 6],
            native_error_pointer: 0,
            message: String::new(), // +1 for terminating zero
        };

        match unsafe {
                  SQLGetDiagRec(H::To::handle_type(),
                                self.handle() as SQLHANDLE,
                                record_number,
                                result.state.as_mut_ptr(),
                                result.native_error_pointer as *mut SQLINTEGER,
                                message.as_mut_ptr(),
                                text_length + 1,
                                null_mut())
              } {
            SQLRETURN::SQL_SUCCESS => {
                message.pop(); //Drop terminating zero
                result.message = String::from_utf8(message).expect("invalid UTF8 encoding");
                Some(result)
            }
            _ => panic!("SQLGetDiagRec returned an unexpected result"),
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use super::super::Raii;

    #[test]
    fn provoke_error() {
        let expected = if cfg!(target_os = "windows") {
            "[Microsoft][ODBC Driver Manager] Invalid argument value"
        } else {
            "[unixODBC][Driver Manager]Invalid use of null pointer"
        };

        use Return;
        use ffi::{SQL_HANDLE_DBC, SQLHANDLE, SQLAllocHandle};

        let environment = match unsafe { Raii::new() } {
            Return::Success(env) => env,
            _ => panic!("unexpected behaviour allocating environment"),
        };

        let error = unsafe {

            // We set the output pointer to zero. This is an error!
            SQLAllocHandle(SQL_HANDLE_DBC,
                           environment.handle() as SQLHANDLE,
                           null_mut());
            // Let's create a diagnostic record describing that error
            environment.get_diag_rec(1).unwrap()
        };

        assert_eq!(error.message, expected);
        assert!(environment.get_diag_rec(2).is_none());
    }
}

