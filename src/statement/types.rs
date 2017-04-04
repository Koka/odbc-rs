use ffi;

/// Implemented for fixed size type those representation is directly compatible with ODBC
pub unsafe trait FixedSizedType: Sized + Default {
    fn c_data_type() -> ffi::SqlCDataType;
}

unsafe impl FixedSizedType for u8 {
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_UTINYINT
    }
}

unsafe impl FixedSizedType for i8 {
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_STINYINT
    }
}

unsafe impl FixedSizedType for i16 {
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_SSHORT
    }
}

unsafe impl FixedSizedType for u16 {
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_USHORT
    }
}

unsafe impl FixedSizedType for i32 {
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_SLONG
    }
}

unsafe impl FixedSizedType for u32 {
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_ULONG
    }
}

unsafe impl FixedSizedType for i64 {
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_SBIGINT
    }
}

unsafe impl FixedSizedType for u64 {
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_UBIGINT
    }
}

unsafe impl FixedSizedType for f32 {
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_FLOAT
    }
}

unsafe impl FixedSizedType for f64 {
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_DOUBLE
    }
}
