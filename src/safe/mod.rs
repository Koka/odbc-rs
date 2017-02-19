//! The safe layer is intended to ensure the basic guarantees of Rust: No memory corruption and no
//! race conditions. It's main purpose is not to provide direct value to the crate user, but to
//! enable the layers on top of it to be written in safe code.

mod diagnostics;
pub use self::diagnostics::*;
mod environment;
pub use self::environment::*;

use raw::{SQLSMALLINT, SQLHANDLE};

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
    fn handle_type() -> SQLSMALLINT;
}