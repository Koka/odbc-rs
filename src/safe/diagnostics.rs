use super::Handle;
use raw::{SQLGetDiagRec, SQLSMALLINT, SQLRETURN, SQLINTEGER};
use std::ptr::null_mut;

/// ODBC Diagonstic record
#[derive(Debug)]
pub struct DiagRec {
    pub state: [u8; 6],
    pub native_error_pointer: i32,
    pub message: String,
}

pub trait GetDiagRec {
    /// Retrieves a diagnostic record
    ///
    /// `record_number` - Record numbers start at one. If you pass an number < 1 the function will
    /// panic. If no record is available for the number specified none is returned.
    fn get_diagnostic_record(&self, record_number: i16) -> Option<DiagRec>;
}

impl<T: Handle> GetDiagRec for T {
    fn get_diagnostic_record(&self, record_number: i16) -> Option<DiagRec> {
        // Call SQLGetDiagRec two times. First time to get the message text length, the second
        // to fill the result with diagnostic information
        let mut text_length: SQLSMALLINT = 0;

        match unsafe {

            SQLGetDiagRec(T::handle_type(),
                          self.handle(),
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
        let mut result = DiagRec {
            state: [0; 6],
            native_error_pointer: 0,
            message: String::new(), // +1 for terminating zero
        };

        match unsafe {
            SQLGetDiagRec(T::handle_type(),
                          self.handle(),
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

    #[test]
    fn provoke_error() {
        use raw::{SQL_HANDLE_DBC, SQLAllocHandle};
        use safe::{Environment, EnvAllocResult};

        let environment = match Environment::new() {
            EnvAllocResult::Success(env) => env,
            _ => panic!("unexpected behaviour allocating environment"),
        };
        let error = unsafe {
            // We set the output pointer to zero. This is an error!
            SQLAllocHandle(SQL_HANDLE_DBC, environment.handle(), null_mut());
            // Let's create a diagnostic record describing that error
            environment.get_diagnostic_record(1).unwrap()
        };
        let expected = if cfg!(target_os = "windows") {
            "[Microsoft][ODBC Driver Manager] Invalid argument value"
        } else {
            "[unixODBC][Driver Manager]Invalid use of null pointer"
        };
        assert_eq!(error.message, expected);
        assert!(environment.get_diagnostic_record(2).is_none());
    }
}