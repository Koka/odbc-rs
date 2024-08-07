use super::{ffi, safe};
use std::error::Error;
use std::ffi::CStr;
use std::{cmp, fmt};

pub const MAX_DIAGNOSTIC_MESSAGE_SIZE: usize = 1024;

/// ODBC Diagnostic Record
///
/// The `description` method of the `std::error::Error` trait only returns the message. Use
/// `std::fmt::Display` to retrieve status code and other information.
pub struct DiagnosticRecord {
    // All elements but the last one, may not be null. The last one must be null.
    state: [ffi::SQLCHAR; ffi::SQL_SQLSTATE_SIZE + 1],
    // Must at least contain one null
    message: [ffi::SQLCHAR; MAX_DIAGNOSTIC_MESSAGE_SIZE],
    // The numbers of characters in message not null
    message_length: ffi::SQLSMALLINT,
    native_error: ffi::SQLINTEGER,
    message_string: String,
}

impl DiagnosticRecord {
    /// get raw state string data.
    pub fn get_raw_state(&self) -> &[u8] {
        &self.state
    }
    /// get raw diagnostics message for avoiding encoding error.
    pub fn get_raw_message(&self) -> &[u8] {
        &self.message[0..self.message_length as usize]
    }
    /// get native odbc error number
    pub fn get_native_error(&self) -> i32 {
        self.native_error
    }
    /// constructs an empty diagnostics message.
    /// this is needed for errors where the driver doesn't return any diagnostics info.
    pub fn empty() -> DiagnosticRecord {
        let message = b"No SQL-driver error information available.";
        let mut rec = DiagnosticRecord {
            state: b"HY000\0".clone(),
            message: [0u8; MAX_DIAGNOSTIC_MESSAGE_SIZE],
            native_error: -1,
            message_length: message.len() as ffi::SQLSMALLINT,
            message_string: String::from(""),
        };
        rec.message[..message.len()].copy_from_slice(message);
        rec
    }
}

impl fmt::Display for DiagnosticRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Todo: replace unwrap with `?` in Rust 1.17
        let state = CStr::from_bytes_with_nul(&self.state).unwrap();

        write!(
            f,
            "State: {}, Native error: {}, Message: {}",
            state.to_str().unwrap(),
            self.native_error,
            self.message_string,
        )
    }
}

impl fmt::Debug for DiagnosticRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl Error for DiagnosticRecord {
    fn description(&self) -> &str {
        &self.message_string
    }
    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}

/// Allows retrieving a diagnostic record, describing errors (or lack thereof) during the last
/// operation.
pub trait GetDiagRec {
    /// Retrieves a diagnostic record
    ///
    /// `record_number` - Record numbers start at one. If you pass an number < 1 the function will
    /// panic. If no record is available for the number specified none is returned.
    fn get_diag_rec(&self, record_number: i16) -> Option<DiagnosticRecord>;
}

impl<D> GetDiagRec for D
where
    D: safe::Diagnostics,
{
    fn get_diag_rec(&self, record_number: i16) -> Option<DiagnosticRecord> {
        use safe::ReturnOption::*;
        let mut message = [0; MAX_DIAGNOSTIC_MESSAGE_SIZE];
        match self.diagnostics(record_number, &mut message) {
            Success(result) | Info(result) => {
                // The message could be larger than the supplied buffer, so we need to limit the message length to the buffer size.
                let mut message_length = cmp::min(result.text_length, MAX_DIAGNOSTIC_MESSAGE_SIZE as ffi::SQLSMALLINT - 1);
                // Some drivers pad the message with null-chars (which is still a valid C string, but not a valid Rust string).
                while message_length > 0 && message[(message_length - 1) as usize] == 0 {
                    message_length = message_length - 1;
                }
                Some(DiagnosticRecord {
                    state: result.state,
                    native_error: result.native_error,
                    message_length,
                    message,
                    message_string: unsafe {
                        ::environment::OS_ENCODING.decode(&message[0..message_length as usize]).0.into_owned()
                    },
                })
            }
            NoData(()) => None,
            Error(()) => panic!("Diagnostics returned error for record number {}. Record numbers have to be at least 1.", record_number),
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    impl DiagnosticRecord {
        fn new() -> DiagnosticRecord {
            DiagnosticRecord {
                state: [0u8; ffi::SQL_SQLSTATE_SIZE + 1],
                message: [0u8; MAX_DIAGNOSTIC_MESSAGE_SIZE],
                native_error: 0,
                message_length: 0,
                message_string: String::from(""),
            }
        }
    }

    #[test]
    fn formatting() {
        // build diagnostic record
        let message = b"[Microsoft][ODBC Driver Manager] Function sequence error\0";
        let mut rec = DiagnosticRecord::new();
        rec.state = b"HY010\0".clone();
        rec.message_string = CStr::from_bytes_with_nul(message)
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        rec.message_length = 56;
        for i in 0..(rec.message_length as usize) {
            rec.message[i] = message[i];
        }

        // test formatting
        assert_eq!(
            format!("{}", rec),
            "State: HY010, Native error: 0, Message: [Microsoft][ODBC Driver Manager] \
             Function sequence error"
        );
    }
}
