//! The safe layer is intended to ensure the basic guarantees of Rust: No memory corruption and no
//! race conditions. It's main purpose is not to provide direct value to the crate user, but to
//! enable the layers on top of it to be written in safe code.

mod diagnostics;
mod environment;
pub use self::diagnostics::*;
pub use self::environment::*;

use std;

use ffi::{SQLSMALLINT, SQLHANDLE, HandleType};

fn as_out_buffer(buffer: &mut [u8]) -> *mut u8 {
    if buffer.len() == 0 {
        std::ptr::null_mut()
    } else {
        buffer.as_mut_ptr()
    }
}

fn as_buffer_length(n: usize) -> SQLSMALLINT {
    use std;
    if n > std::i16::MAX as usize {
        std::i16::MAX
    } else {
        n as i16
    }
}

pub unsafe trait Handle {
    fn handle(&self) -> SQLHANDLE;
    fn handle_type() -> HandleType;
}