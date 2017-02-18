mod diag_rec;
pub use self::diag_rec::*;
mod environment;
pub use self::environment::*;

use raw::{SQLSMALLINT, SQLHANDLE};

pub unsafe trait Handle {
    fn handle(&self) -> SQLHANDLE;
    fn handle_type() -> SQLSMALLINT;
}