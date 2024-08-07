use super::{ffi, safe, DiagnosticRecord, GetDiagRec, Handle, OdbcObject, Return};
use std::marker::PhantomData;
use std::ptr::null_mut;

/// Wrapper around handle types which ensures the wrapped value is always valid.
///
/// Resource Acquisition Is Initialization
#[derive(Debug)]
pub struct Raii<'p, T: OdbcObject> {
    //Invariant: Should always point to a valid odbc Object
    handle: *mut T,
    // we use phantom data to tell the borrow checker that we need to keep the data source alive
    // for the lifetime of the handle
    parent: PhantomData<&'p ()>,
}

impl<'p, T: OdbcObject> Handle for Raii<'p, T> {
    type To = T;
    unsafe fn handle(&self) -> *mut T {
        self.handle
    }
}

unsafe impl<'p, T: OdbcObject> safe::Handle for Raii<'p, T> {
    const HANDLE_TYPE: ffi::HandleType = T::HANDLE_TYPE;

    fn handle(&self) -> ffi::SQLHANDLE {
        self.handle as ffi::SQLHANDLE
    }
}

impl<'p, T: OdbcObject> Drop for Raii<'p, T> {
    fn drop(&mut self) {
        match unsafe { ffi::SQLFreeHandle(T::HANDLE_TYPE, self.handle() as ffi::SQLHANDLE) } {
            ffi::SQL_SUCCESS => (),
            ffi::SQL_ERROR => {
                let rec = self.get_diag_rec(1).unwrap_or_else(DiagnosticRecord::empty);
                error!("Error freeing handle: {}", rec)
            }
            _ => panic!("Unexepected return value of SQLFreeHandle"),
        }
    }
}

impl<'p, T: OdbcObject> Raii<'p, T> {
    pub fn with_parent<P>(parent: &'p P) -> Return<Self>
    where
        P: Handle<To = T::Parent>,
    {
        let mut handle: ffi::SQLHANDLE = null_mut();
        match unsafe {
            ffi::SQLAllocHandle(
                T::HANDLE_TYPE,
                parent.handle() as ffi::SQLHANDLE,
                &mut handle as *mut ffi::SQLHANDLE,
            )
        } {
            ffi::SQL_SUCCESS => Return::Success(Raii {
                handle: handle as *mut T,
                parent: PhantomData,
            }),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(Raii {
                handle: handle as *mut T,
                parent: PhantomData,
            }),
            ffi::SQL_ERROR => Return::Error,
            _ => panic!("SQLAllocHandle returned unexpected result"),
        }
    }
}
