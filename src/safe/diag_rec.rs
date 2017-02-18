use super::Handle;
use raw::{SQLGetDiagRec, SQLSMALLINT, SQLRETURN, SQLINTEGER};
use std::ptr::null_mut;

#[derive(Debug)]
pub struct DiagRec {
    pub state: [u8; 6],
    pub native_error_pointer: i32,
    pub message: String,
}

pub fn get_diag_rec<H: Handle>(wrapper: &H, record_number: i16) -> Option<DiagRec> {

    // Call SQLGetDiagRec two times. First time to get the message text length, the second
    // to fill the result with diagnostic information
    let mut text_length: SQLSMALLINT = 0;

    match unsafe {

        SQLGetDiagRec(H::handle_type(),
                      wrapper.handle(),
                      record_number,
                      null_mut(),
                      null_mut(),
                      null_mut(),
                      0,
                      &mut text_length as *mut SQLSMALLINT)
    } {
        SQLRETURN::SQL_SUCCESS |
        SQLRETURN::SQL_SUCCESS_WITH_INFO => {
            let mut message = vec![0; (text_length + 1) as usize];
            let mut result = DiagRec {
                state: [0; 6],
                native_error_pointer: 0,
                message: String::new(), // +1 for terminating zero
            };

            match unsafe {
                SQLGetDiagRec(H::handle_type(),
                              wrapper.handle(),
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
        SQLRETURN::SQL_ERROR => {
            if record_number > 0 {
                panic!("SQLGetDiagRec returned an unexpected result")
            } else {
                panic!("record number start at 1 has been {}", record_number)
            }
        }
        SQLRETURN::SQL_NO_DATA => None,
        _ => panic!("SQLGetDiagRec returned an unexpected result"),
    }
}