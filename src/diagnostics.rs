use super::{ffi, Handle, OdbcObject};
use std::fmt;
use std::ffi::CStr;
use std::error::Error;

/// ODBC Diagnostic Record
///
/// The `description` method of the `std::error::Error` trait only returns the message. Use
/// `std::fmt::Display` to retrive status code and other information.
pub struct DiagnosticRecord {
    // All elements but the last one, may not be nul. The last one must be nul.
    state: [ffi::SQLCHAR; ffi::SQL_SQLSTATE_SIZE + 1],
    // Must at least contain one nul
    message: [ffi::SQLCHAR; ffi::SQL_MAX_MESSAGE_LENGTH as usize],
    // The numbers of characters in message not nul
    message_length: ffi::SQLSMALLINT,
    native_error: ffi::SQLINTEGER,
}

impl DiagnosticRecord {
    fn new() -> DiagnosticRecord {
        DiagnosticRecord {
            state: [0u8; ffi::SQL_SQLSTATE_SIZE + 1],
            message: [0u8; ffi::SQL_MAX_MESSAGE_LENGTH as usize],
            native_error: 0,
            message_length: 0,
        }
    }
}

impl fmt::Display for DiagnosticRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Todo: replace unwrap with `?` in Rust 1.17
        let state = CStr::from_bytes_with_nul(&self.state).unwrap();
        let message = CStr::from_bytes_with_nul(&self.message[0..
                                                 (self.message_length as usize + 1)])
                .unwrap();

        write!(f,
               "State: {}, Native error: {}, Message: {}",
               state.to_str().unwrap(),
               self.native_error,
               message.to_str().unwrap())
    }
}

impl fmt::Debug for DiagnosticRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl Error for DiagnosticRecord {
    fn description(&self) -> &str {
        CStr::from_bytes_with_nul(&self.message[0..(self.message_length as usize + 1)])
            .unwrap()
            .to_str()
            .unwrap()
    }
    fn cause(&self) -> Option<&Error> {
        None
    }
}

/// Allows retriving a diagnostic record, describing errors (or lack thereof) during the last
/// operation.
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
    fn get_diag_rec(&self, record_number: i16) -> Option<(DiagnosticRecord)> {
        let mut result = DiagnosticRecord::new();

        match unsafe {
                  ffi::SQLGetDiagRec(H::To::handle_type(),
                                     self.handle() as ffi::SQLHANDLE,
                                     record_number,
                                     result.state.as_mut_ptr(),
                                     &mut result.native_error as *mut ffi::SQLINTEGER,
                                     result.message.as_mut_ptr(),
                                     ffi::SQL_MAX_MESSAGE_LENGTH,
                                     &mut result.message_length as *mut ffi::SQLSMALLINT)
              } {
            ffi::SQL_SUCCESS => Some(result),
            ffi::SQL_NO_DATA => None,
            ffi::SQL_SUCCESS_WITH_INFO => Some(result),
            ffi::SQL_ERROR => {
                if record_number > 0 {
                    panic!("SQLGetDiagRec returned SQL_ERROR")
                } else {
                    panic!("record number start at 1 has been {}", record_number)
                }
            }
            _ => panic!("SQLGetDiag returned unexpected result"),
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn formatting() {

        // build diagnostic record
        let message = b"[Microsoft][ODBC Driver Manager] Function sequence error\0";
        let mut rec = DiagnosticRecord::new();
        rec.state = b"HY010\0".clone();
        rec.message_length = 56;
        for i in 0..(rec.message_length as usize) {
            rec.message[i] = message[i];
        }

        // test formatting
        assert_eq!(format!("{}", rec),
                   "State: HY010, Native error: 0, Message: [Microsoft][ODBC Driver Manager] \
                    Function sequence error");
    }
}

