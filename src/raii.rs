use super::{ffi, OdbcObject, GetDiagRec, Return, Handle};
use std::ptr::null_mut;

/// Wrapper around handle types which ensures the wrapped value is always valid.
///
/// Resource Acquisition Is Initialization
pub struct Raii<T: OdbcObject> {
    //Invariant: Should always point to a valid odbc Object
    handle: *mut T,
}

impl<T: OdbcObject> Handle for Raii<T> {
    type To = T;
    unsafe fn handle(&self) -> *mut T {
        self.handle
    }
}

impl<T: OdbcObject> Drop for Raii<T> {
    fn drop(&mut self) {
        match unsafe { ffi::SQLFreeHandle(T::handle_type(), self.handle() as ffi::SQLHANDLE) } {
            ffi::SQL_SUCCESS => (),
            ffi::SQL_ERROR => error!("Error freeing handle: {}", self.get_diag_rec(1).unwrap()),
            _ => panic!("Unexepected return value of SQLFreeHandle"),
        }
    }
}

impl<T: OdbcObject> Raii<T> {
    pub fn with_parent<P>(parent: &P) -> Return<Self>
        where P: Handle<To = T::Parent>
    {
        let mut handle: ffi::SQLHANDLE = null_mut();
        match unsafe {
            ffi::SQLAllocHandle(T::handle_type(),
                                parent.handle() as ffi::SQLHANDLE,
                                &mut handle as *mut ffi::SQLHANDLE)
        } {
            ffi::SQL_SUCCESS => Return::Success(Raii { handle: handle as *mut T }),
            ffi::SQL_SUCCESS_WITH_INFO => {
                Return::SuccessWithInfo(Raii { handle: handle as *mut T })
            }
            ffi::SQL_ERROR => Return::Error,
            _ => panic!("SQLAllocHandle returned unexpected result"),
        }
    }
}

impl Raii<ffi::Env> {
    pub unsafe fn new() -> Return<Self> {
        let mut handle: ffi::SQLHANDLE = null_mut();
        match ffi::SQLAllocHandle(ffi::Env::handle_type(),
                                  null_mut(),
                                  &mut handle as *mut ffi::SQLHANDLE) {
            ffi::SQL_SUCCESS => Return::Success(Raii { handle: handle as ffi::SQLHENV }),
            ffi::SQL_SUCCESS_WITH_INFO => {
                Return::SuccessWithInfo(Raii { handle: handle as ffi::SQLHENV })
            }
            ffi::SQL_ERROR => Return::Error,
            _ => panic!("SQLAllocHandle returned unexpected result"),
        }
    }
}
