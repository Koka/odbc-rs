use ffi;

/// Trait to be implemented by all opaque types which are referenced by handles in the ffi layer
pub unsafe trait OdbcObject {
    const HANDLE_TYPE: ffi::HandleType;
    type Parent;
}

unsafe impl OdbcObject for ffi::Env {
    const HANDLE_TYPE: ffi::HandleType = ffi::SQL_HANDLE_ENV;
    type Parent = ();
}

unsafe impl OdbcObject for ffi::Dbc {
    const HANDLE_TYPE: ffi::HandleType = ffi::SQL_HANDLE_DBC;
    type Parent = ffi::Env;
}

unsafe impl OdbcObject for ffi::Stmt {
    const HANDLE_TYPE: ffi::HandleType = ffi::SQL_HANDLE_STMT;
    type Parent = ffi::Dbc;
}
