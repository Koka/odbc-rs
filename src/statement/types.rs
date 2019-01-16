use ffi;
use std::str::from_utf8;
use std::slice::from_raw_parts;
use std::mem::{size_of, transmute};
use std::ffi::CString;

pub unsafe trait OdbcType<'a>: Sized {
    fn sql_data_type() -> ffi::SqlDataType;
    fn c_data_type() -> ffi::SqlCDataType;
    fn convert(_: &'a [u8]) -> Self;
    fn column_size(&self) -> ffi::SQLULEN;
    fn value_ptr(&self) -> ffi::SQLPOINTER;
    fn decimal_digits(&self) -> ffi::SQLSMALLINT {
        0
    }
}

unsafe impl<'a> OdbcType<'a> for &'a[u8] {
    fn sql_data_type() -> ffi::SqlDataType {
        ffi::SQL_EXT_VARBINARY
    }
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_BINARY
    }

    fn convert(buffer: &'a [u8]) -> Self {
        buffer
    }

    fn column_size(&self) -> ffi::SQLULEN {
        self.len() as ffi::SQLULEN
    }

    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self.as_ptr() as *const Self as ffi::SQLPOINTER
    }
}

unsafe impl<'a> OdbcType<'a> for Vec<u8> {
    fn sql_data_type() -> ffi::SqlDataType {
        ffi::SQL_EXT_VARBINARY
    }
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_BINARY
    }

    fn convert(buffer: &'a [u8]) -> Self {
        buffer.to_vec()
    }

    fn column_size(&self) -> ffi::SQLULEN {
        self.len() as ffi::SQLULEN
    }

    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self.as_ptr() as *const Self as ffi::SQLPOINTER
    }
}

unsafe impl<'a> OdbcType<'a> for &'a[u16] {
    fn sql_data_type() -> ffi::SqlDataType {
        ffi::SQL_EXT_WVARCHAR
    }
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_WCHAR
    }

    fn convert(buffer: &'a [u8]) -> Self {
        unsafe { from_raw_parts(buffer.as_ptr() as *const u16, buffer.len() / 2) }
    }

    fn column_size(&self) -> ffi::SQLULEN {
        self.len() as ffi::SQLULEN
    }

    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self.as_ptr() as *const Self as ffi::SQLPOINTER
    }
}

unsafe impl<'a> OdbcType<'a> for Vec<u16> {
    fn sql_data_type() -> ffi::SqlDataType {
        ffi::SQL_EXT_WVARCHAR
    }
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_WCHAR
    }

    fn convert(buffer: &'a [u8]) -> Self {
        let buffer = unsafe { from_raw_parts(buffer.as_ptr() as *const u16, buffer.len() / 2) };
        buffer.to_vec()
    }

    fn column_size(&self) -> ffi::SQLULEN {
        self.len() as ffi::SQLULEN
    }

    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self.as_ptr() as *const Self as ffi::SQLPOINTER
    }
}

unsafe impl<'a> OdbcType<'a> for CString {
    fn sql_data_type() -> ffi::SqlDataType {
        ffi::SQL_VARCHAR
    }
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_CHAR
    }

    fn convert(buffer: &'a [u8]) -> Self {
        CString::new(buffer).unwrap()
    }

    fn column_size(&self) -> ffi::SQLULEN {
        self.as_bytes().len() as ffi::SQLULEN
    }

    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self.as_bytes().as_ptr() as *const Self as ffi::SQLPOINTER
    }
}

unsafe impl<'a> OdbcType<'a> for String {
    fn sql_data_type() -> ffi::SqlDataType {
        ffi::SQL_VARCHAR
    }
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_CHAR
    }

    fn convert(buffer: &'a [u8]) -> Self {
        from_utf8(buffer).unwrap().to_owned()
    }

    fn column_size(&self) -> ffi::SQLULEN {
        self.as_bytes().len() as ffi::SQLULEN
    }

    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self.as_bytes().as_ptr() as *const Self as ffi::SQLPOINTER
    }
}

unsafe impl<'a> OdbcType<'a> for &'a str {
    fn sql_data_type() -> ffi::SqlDataType {
        ffi::SQL_VARCHAR
    }
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_CHAR
    }

    fn convert(buffer: &'a [u8]) -> Self {
        from_utf8(buffer).unwrap()
    }

    fn column_size(&self) -> ffi::SQLULEN {
        self.as_bytes().len() as ffi::SQLULEN
    }

    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self.as_bytes().as_ptr() as *const Self as ffi::SQLPOINTER
    }
}

fn convert_primitive<T>(buf: &[u8]) -> T
where
    T: Copy,
{
    unsafe { from_raw_parts(buf.as_ptr() as *const T, 1)[0] }
}

unsafe impl<'a> OdbcType<'a> for u8 {
    fn sql_data_type() -> ffi::SqlDataType {
        ffi::SQL_SMALLINT
    }
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_UTINYINT
    }

    fn convert(buffer: &'a [u8]) -> Self {
        convert_primitive(buffer)
    }

    fn column_size(&self) -> ffi::SQLULEN {
        size_of::<Self>() as ffi::SQLULEN
    }
    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self as *const Self as ffi::SQLPOINTER
    }
}

unsafe impl<'a> OdbcType<'a> for i8 {
    fn sql_data_type() -> ffi::SqlDataType {
        ffi::SQL_SMALLINT
    }
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_STINYINT
    }

    fn convert(buffer: &'a [u8]) -> Self {
        convert_primitive(buffer)
    }

    fn column_size(&self) -> ffi::SQLULEN {
        size_of::<Self>() as ffi::SQLULEN
    }
    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self as *const Self as ffi::SQLPOINTER
    }
}

unsafe impl<'a> OdbcType<'a> for i16 {
    fn sql_data_type() -> ffi::SqlDataType {
        ffi::SQL_SMALLINT
    }
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_SSHORT
    }

    fn convert(buffer: &'a [u8]) -> Self {
        convert_primitive(buffer)
    }

    fn column_size(&self) -> ffi::SQLULEN {
        size_of::<Self>() as ffi::SQLULEN
    }
    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self as *const Self as ffi::SQLPOINTER
    }
}

unsafe impl<'a> OdbcType<'a> for u16 {
    fn sql_data_type() -> ffi::SqlDataType {
        ffi::SQL_SMALLINT
    }
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_USHORT
    }

    fn convert(buffer: &'a [u8]) -> Self {
        convert_primitive(buffer)
    }

    fn column_size(&self) -> ffi::SQLULEN {
        size_of::<Self>() as ffi::SQLULEN
    }
    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self as *const Self as ffi::SQLPOINTER
    }
}

unsafe impl<'a> OdbcType<'a> for i32 {
    fn sql_data_type() -> ffi::SqlDataType {
        ffi::SQL_INTEGER
    }
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_SLONG
    }

    fn convert(buffer: &'a [u8]) -> Self {
        convert_primitive(buffer)
    }

    fn column_size(&self) -> ffi::SQLULEN {
        size_of::<Self>() as ffi::SQLULEN
    }
    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self as *const Self as ffi::SQLPOINTER
    }
}

unsafe impl<'a> OdbcType<'a> for u32 {
    fn sql_data_type() -> ffi::SqlDataType {
        ffi::SQL_INTEGER
    }
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_ULONG
    }

    fn convert(buffer: &'a [u8]) -> Self {
        convert_primitive(buffer)
    }

    fn column_size(&self) -> ffi::SQLULEN {
        size_of::<Self>() as ffi::SQLULEN
    }
    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self as *const Self as ffi::SQLPOINTER
    }
}

unsafe impl<'a> OdbcType<'a> for i64 {
    fn sql_data_type() -> ffi::SqlDataType {
        ffi::SQL_INTEGER
    }
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_SBIGINT
    }

    fn convert(buffer: &'a [u8]) -> Self {
        convert_primitive(buffer)
    }

    fn column_size(&self) -> ffi::SQLULEN {
        size_of::<Self>() as ffi::SQLULEN
    }
    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self as *const Self as ffi::SQLPOINTER
    }
}

unsafe impl<'a> OdbcType<'a> for u64 {
    fn sql_data_type() -> ffi::SqlDataType {
        ffi::SQL_INTEGER
    }
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_UBIGINT
    }

    fn convert(buffer: &'a [u8]) -> Self {
        convert_primitive(buffer)
    }

    fn column_size(&self) -> ffi::SQLULEN {
        size_of::<Self>() as ffi::SQLULEN
    }
    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self as *const Self as ffi::SQLPOINTER
    }
}

unsafe impl<'a> OdbcType<'a> for f32 {
    fn sql_data_type() -> ffi::SqlDataType {
        ffi::SQL_FLOAT
    }
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_FLOAT
    }

    fn convert(buffer: &'a [u8]) -> Self {
        convert_primitive(buffer)
    }

    fn column_size(&self) -> ffi::SQLULEN {
        size_of::<Self>() as ffi::SQLULEN
    }
    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self as *const Self as ffi::SQLPOINTER
    }
}

unsafe impl<'a> OdbcType<'a> for f64 {
    fn sql_data_type() -> ffi::SqlDataType {
        ffi::SQL_DOUBLE
    }
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_DOUBLE
    }

    fn convert(buffer: &'a [u8]) -> Self {
        convert_primitive(buffer)
    }

    fn column_size(&self) -> ffi::SQLULEN {
        size_of::<Self>() as ffi::SQLULEN
    }
    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self as *const Self as ffi::SQLPOINTER
    }
}

unsafe impl<'a> OdbcType<'a> for bool {
    fn sql_data_type() -> ffi::SqlDataType {
        ffi::SQL_EXT_BIT
    }
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_BIT
    }

    fn convert(buffer: &'a [u8]) -> Self {
        assert!(buffer.len() == 1);
        buffer[0] > 0
    }

    fn column_size(&self) -> ffi::SQLULEN {
        1 as ffi::SQLULEN
    }
    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self as *const Self as ffi::SQLPOINTER
    }
}

pub type SqlDate = ffi::SQL_DATE_STRUCT;

unsafe impl<'a> OdbcType<'a> for SqlDate {
    fn sql_data_type() -> ffi::SqlDataType {
        ffi::SQL_DATE
    }
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_DATE
    }

    fn convert(buffer: &'a [u8]) -> Self {
        assert_eq!(buffer.len(), size_of::<Self>());
        unsafe {
            let ptr = buffer.as_ptr() as *const [u8; size_of::<Self>()];
            transmute(*ptr)
        }
    }

    fn column_size(&self) -> ffi::SQLULEN {
        size_of::<Self>() as ffi::SQLULEN
    }
    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self as *const Self as ffi::SQLPOINTER
    }
}

pub type SqlTime = ffi::SQL_TIME_STRUCT;

unsafe impl<'a> OdbcType<'a> for SqlTime {
    fn sql_data_type() -> ffi::SqlDataType {
        ffi::SQL_TIME
    }
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_TIME
    }

    fn convert(buffer: &'a [u8]) -> Self {
        assert_eq!(buffer.len(), size_of::<Self>());
        unsafe {
            let ptr = buffer.as_ptr() as *const [u8; size_of::<Self>()];
            transmute(*ptr)
        }
    }

    fn column_size(&self) -> ffi::SQLULEN {
        size_of::<Self>() as ffi::SQLULEN
    }
    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self as *const Self as ffi::SQLPOINTER
    }
}

pub type SqlTimestamp = ffi::SQL_TIMESTAMP_STRUCT;

unsafe impl<'a> OdbcType<'a> for SqlTimestamp {
    fn sql_data_type() -> ffi::SqlDataType {
        ffi::SQL_TIMESTAMP
    }
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_TYPE_TIMESTAMP
    }

    fn convert(buffer: &'a [u8]) -> Self {
        assert_eq!(buffer.len(), size_of::<Self>());
        unsafe {
            let ptr = buffer.as_ptr() as *const [u8; size_of::<Self>()];
            transmute(*ptr)
        }
    }

    fn column_size(&self) -> ffi::SQLULEN {
        size_of::<Self>() as ffi::SQLULEN
    }
    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self as *const Self as ffi::SQLPOINTER
    }
}

pub type SqlSsTime2 = ffi::SQL_SS_TIME2_STRUCT;

unsafe impl<'a> OdbcType<'a> for SqlSsTime2 {
    fn sql_data_type() -> ffi::SqlDataType {
        ffi::SQL_EXT_VARBINARY
    }
    fn c_data_type() -> ffi::SqlCDataType {
        // NOTE: ODBC 3.5 and earlier
        ffi::SQL_C_BINARY
    }

    fn convert(buffer: &'a [u8]) -> Self {
        assert_eq!(buffer.len(), size_of::<Self>());
        unsafe {
            let ptr = buffer.as_ptr() as *const [u8; size_of::<Self>()];
            transmute(*ptr)
        }
    }

    fn column_size(&self) -> ffi::SQLULEN {
        size_of::<Self>() as ffi::SQLULEN
    }
    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self as *const Self as ffi::SQLPOINTER
    }
}

unsafe impl<'a, T> OdbcType<'a> for Option<T> where T: OdbcType<'a> {
    fn sql_data_type() -> ffi::SqlDataType {
        T::sql_data_type()
    }
    fn c_data_type() -> ffi::SqlCDataType {
        T::c_data_type()
    }

    fn convert(buffer: &'a [u8]) -> Self {
        Some(T::convert(buffer))
    }

    fn column_size(&self) -> ffi::SQLULEN {
        if let Some(t) = self {
            t.column_size()
        } else {
            0
        }
    }
    fn value_ptr(&self) -> ffi::SQLPOINTER {
        if let Some(t) = self {
            t.value_ptr()
        } else {
            0 as *const Self as ffi::SQLPOINTER
        }
    }
}