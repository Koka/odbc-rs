use ffi;
use std::str::from_utf8;
use std::slice::from_raw_parts;
use std::mem::size_of;

pub trait OdbcType<'a>: Sized {
    fn c_data_type() -> ffi::SqlCDataType;
    fn convert(_: &'a [u8]) -> Self {
        unreachable!()
    }
    fn column_size(&self) -> ffi::SQLULEN { unreachable!() }
    fn value_ptr(&self) -> ffi::SQLPOINTER { unreachable!() }
    fn decimal_digits(&self) -> ffi::SQLSMALLINT {
        0
    }
}

impl<'a> OdbcType<'a> for String {
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_CHAR
    }

    fn convert(buffer: &'a [u8]) -> Self {
        from_utf8(buffer)
            .unwrap()
            .to_owned()
    }

    fn column_size(&self) -> ffi::SQLULEN {
        self.as_bytes().len() as ffi::SQLULEN
    }

    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self.as_bytes().as_ptr() as *const Self as ffi::SQLPOINTER
    }
}

impl<'a> OdbcType<'a> for &'a str {
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_CHAR
    }

    fn convert(buffer: &'a [u8]) -> Self {
        from_utf8(buffer)
            .unwrap()
    }

    fn column_size(&self) -> ffi::SQLULEN {
        self.as_bytes().len() as ffi::SQLULEN
    }

    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self.as_bytes().as_ptr() as *const Self as ffi::SQLPOINTER
    }
}

fn convert_primitive<T>(buf: &[u8]) -> T
    where T: Copy
{
    unsafe {
        from_raw_parts(buf.as_ptr() as *const T, 1)[0]
    }
}

impl<'a> OdbcType<'a> for u8 {
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_UTINYINT
    }

    fn convert(buffer: &'a [u8]) -> Self {
        convert_primitive(buffer)
    }

    fn column_size(&self) -> ffi::SQLULEN { size_of::<Self>() as ffi::SQLULEN }
    fn value_ptr(&self) -> ffi::SQLPOINTER { self as *const Self as ffi::SQLPOINTER }
}

impl<'a> OdbcType<'a> for i8 {
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_STINYINT
    }

    fn convert(buffer: &'a [u8]) -> Self {
        convert_primitive(buffer)
    }

    fn column_size(&self) -> ffi::SQLULEN { size_of::<Self>() as ffi::SQLULEN }
    fn value_ptr(&self) -> ffi::SQLPOINTER { self as *const Self as ffi::SQLPOINTER }
}

impl<'a> OdbcType<'a> for i16 {
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_SSHORT
    }

    fn convert(buffer: &'a [u8]) -> Self {
        convert_primitive(buffer)
    }

    fn column_size(&self) -> ffi::SQLULEN { size_of::<Self>() as ffi::SQLULEN }
    fn value_ptr(&self) -> ffi::SQLPOINTER { self as *const Self as ffi::SQLPOINTER }
}

impl<'a> OdbcType<'a> for u16 {
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_USHORT
    }

    fn convert(buffer: &'a [u8]) -> Self {
        convert_primitive(buffer)
    }

    fn column_size(&self) -> ffi::SQLULEN { size_of::<Self>() as ffi::SQLULEN }
    fn value_ptr(&self) -> ffi::SQLPOINTER { self as *const Self as ffi::SQLPOINTER }
}

impl<'a> OdbcType<'a> for i32 {
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_SLONG
    }

    fn convert(buffer: &'a [u8]) -> Self {
        convert_primitive(buffer)
    }

    fn column_size(&self) -> ffi::SQLULEN { size_of::<Self>() as ffi::SQLULEN }
    fn value_ptr(&self) -> ffi::SQLPOINTER { self as *const Self as ffi::SQLPOINTER }
}

impl<'a> OdbcType<'a> for u32 {
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_ULONG
    }

    fn convert(buffer: &'a [u8]) -> Self {
        convert_primitive(buffer)
    }

    fn column_size(&self) -> ffi::SQLULEN { size_of::<Self>() as ffi::SQLULEN }
    fn value_ptr(&self) -> ffi::SQLPOINTER { self as *const Self as ffi::SQLPOINTER }
}

impl<'a> OdbcType<'a> for i64 {
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_SBIGINT
    }

    fn convert(buffer: &'a [u8]) -> Self {
        convert_primitive(buffer)
    }

    fn column_size(&self) -> ffi::SQLULEN { size_of::<Self>() as ffi::SQLULEN }
    fn value_ptr(&self) -> ffi::SQLPOINTER { self as *const Self as ffi::SQLPOINTER }
}

impl<'a> OdbcType<'a> for u64 {
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_UBIGINT
    }

    fn convert(buffer: &'a [u8]) -> Self {
        convert_primitive(buffer)
    }

    fn column_size(&self) -> ffi::SQLULEN { size_of::<Self>() as ffi::SQLULEN }
    fn value_ptr(&self) -> ffi::SQLPOINTER { self as *const Self as ffi::SQLPOINTER }
}

impl<'a> OdbcType<'a> for f32 {
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_FLOAT
    }

    fn convert(buffer: &'a [u8]) -> Self {
        convert_primitive(buffer)
    }

    fn column_size(&self) -> ffi::SQLULEN { size_of::<Self>() as ffi::SQLULEN }
    fn value_ptr(&self) -> ffi::SQLPOINTER { self as *const Self as ffi::SQLPOINTER }
}

impl<'a> OdbcType<'a> for f64 {
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_DOUBLE
    }

    fn convert(buffer: &'a [u8]) -> Self {
        convert_primitive(buffer)
    }

    fn column_size(&self) -> ffi::SQLULEN { size_of::<Self>() as ffi::SQLULEN }
    fn value_ptr(&self) -> ffi::SQLPOINTER { self as *const Self as ffi::SQLPOINTER }
}
