use ffi;

/// Trait to be implemented by all opaque types which are referenced by handles in the ffi layer
pub unsafe trait OdbcObject {
    type Parent;
    fn handle_type() -> ffi::HandleType;
}

unsafe impl OdbcObject for ffi::Env {
    type Parent = ();
    fn handle_type() -> ffi::HandleType {
        ffi::SQL_HANDLE_ENV
    }
}

unsafe impl OdbcObject for ffi::Dbc {
    type Parent = ffi::Env;
    fn handle_type() -> ffi::HandleType {
        ffi::SQL_HANDLE_DBC
    }
}

unsafe impl OdbcObject for ffi::Stmt {
    type Parent = ffi::Dbc;
    fn handle_type() -> ffi::HandleType {
        ffi::SQL_HANDLE_STMT
    }
}
