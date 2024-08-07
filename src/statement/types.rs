use ffi;
use std::borrow::Cow::{Borrowed, Owned};
use std::ffi::CString;
use std::mem::{size_of, transmute};
use std::slice::from_raw_parts;

pub struct EncodedValue {
    pub buf: Option<Vec<u8>>,
}

impl EncodedValue {
    pub fn new(buf: Option<Vec<u8>>) -> Self {
        Self { buf }
    }

    pub fn has_value(&self) -> bool {
        self.buf.is_some()
    }

    pub fn column_size(&self) -> ffi::SQLULEN {
        if let Some(buf) = &self.buf {
            buf.len() as ffi::SQLULEN
        } else {
            0
        }
    }

    pub fn value_ptr(&self) -> ffi::SQLPOINTER {
        if let Some(buf) = &self.buf {
            buf.as_ptr() as *const Self as ffi::SQLPOINTER
        } else {
            0 as *const Self as ffi::SQLPOINTER
        }
    }
}

pub unsafe trait OdbcType<'a>: Sized {
    fn sql_data_type() -> ffi::SqlDataType;
    fn c_data_type() -> ffi::SqlCDataType;
    fn convert(_: &'a [u8]) -> Self;
    fn column_size(&self) -> ffi::SQLULEN;
    fn null_bytes_count() -> usize {
        0
    }
    fn value_ptr(&self) -> ffi::SQLPOINTER;
    fn decimal_digits(&self) -> ffi::SQLSMALLINT {
        0
    }
    fn encoded_value(&self) -> EncodedValue;
}

unsafe impl<'a> OdbcType<'a> for &'a [u8] {
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

    fn encoded_value(&self) -> EncodedValue {
        EncodedValue::new(None)
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

    fn encoded_value(&self) -> EncodedValue {
        EncodedValue::new(None)
    }
}

unsafe impl<'a> OdbcType<'a> for &'a [u16] {
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
        (self.len() * 2) as ffi::SQLULEN
    }

    fn null_bytes_count() -> usize {
        2
    }

    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self.as_ptr() as *const Self as ffi::SQLPOINTER
    }

    fn encoded_value(&self) -> EncodedValue {
        EncodedValue::new(None)
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
        (self.len() * 2) as ffi::SQLULEN
    }

    fn null_bytes_count() -> usize {
        2
    }

    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self.as_ptr() as *const Self as ffi::SQLPOINTER
    }

    fn encoded_value(&self) -> EncodedValue {
        EncodedValue::new(None)
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

    fn null_bytes_count() -> usize {
        1
    }

    fn encoded_value(&self) -> EncodedValue {
        EncodedValue::new(None)
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
        unsafe { ::environment::DB_ENCODING }
            .decode(buffer)
            .0
            .to_string()
    }

    fn column_size(&self) -> ffi::SQLULEN {
        unsafe { ::environment::DB_ENCODING }.encode(&self).0.len() as ffi::SQLULEN
    }

    fn value_ptr(&self) -> ffi::SQLPOINTER {
        unsafe { ::environment::DB_ENCODING }
            .encode(&self)
            .0
            .as_ptr() as *const Self as ffi::SQLPOINTER
    }

    fn null_bytes_count() -> usize {
        1
    }

    fn encoded_value(&self) -> EncodedValue {
        EncodedValue::new(Some(
            unsafe { ::environment::DB_ENCODING }
                .encode(&self)
                .0
                .to_vec(),
        ))
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
        let cow = unsafe { ::environment::DB_ENCODING }.decode(buffer).0;
        match cow {
            Borrowed(strref) => strref,
            Owned(_string) => {
                panic!("Couldn't convert data to `&str`. Try `String` or `Cow<str>` instead.")
            }
        }
    }

    fn column_size(&self) -> ffi::SQLULEN {
        unsafe { ::environment::DB_ENCODING }.encode(self).0.len() as ffi::SQLULEN
    }

    fn value_ptr(&self) -> ffi::SQLPOINTER {
        unsafe { ::environment::DB_ENCODING }
            .encode(self)
            .0
            .as_ptr() as *const Self as ffi::SQLPOINTER
    }

    fn null_bytes_count() -> usize {
        1
    }

    fn encoded_value(&self) -> EncodedValue {
        EncodedValue::new(Some(
            unsafe { ::environment::DB_ENCODING }
                .encode(&self)
                .0
                .to_vec(),
        ))
    }
}

unsafe impl<'a> OdbcType<'a> for ::std::borrow::Cow<'a, str> {
    fn sql_data_type() -> ffi::SqlDataType {
        ffi::SQL_VARCHAR
    }
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_CHAR
    }

    fn convert(buffer: &'a [u8]) -> Self {
        unsafe { ::environment::DB_ENCODING.decode(buffer).0 }
    }

    fn column_size(&self) -> ffi::SQLULEN {
        unsafe { ::environment::DB_ENCODING }.encode(self).0.len() as ffi::SQLULEN
    }

    fn value_ptr(&self) -> ffi::SQLPOINTER {
        unsafe { ::environment::DB_ENCODING }
            .encode(self)
            .0
            .as_ptr() as *const Self as ffi::SQLPOINTER
    }

    fn null_bytes_count() -> usize {
        1
    }

    fn encoded_value(&self) -> EncodedValue {
        EncodedValue::new(Some(
            unsafe { ::environment::DB_ENCODING }
                .encode(self)
                .0
                .to_vec(),
        ))
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

    fn encoded_value(&self) -> EncodedValue {
        EncodedValue::new(None)
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

    fn encoded_value(&self) -> EncodedValue {
        EncodedValue::new(None)
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

    fn encoded_value(&self) -> EncodedValue {
        EncodedValue::new(None)
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

    fn encoded_value(&self) -> EncodedValue {
        EncodedValue::new(None)
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

    fn encoded_value(&self) -> EncodedValue {
        EncodedValue::new(None)
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

    fn encoded_value(&self) -> EncodedValue {
        EncodedValue::new(None)
    }
}

unsafe impl<'a> OdbcType<'a> for i64 {
    fn sql_data_type() -> ffi::SqlDataType {
        ffi::SQL_EXT_BIGINT
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

    fn encoded_value(&self) -> EncodedValue {
        EncodedValue::new(None)
    }
}

unsafe impl<'a> OdbcType<'a> for u64 {
    fn sql_data_type() -> ffi::SqlDataType {
        ffi::SQL_EXT_BIGINT
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

    fn encoded_value(&self) -> EncodedValue {
        EncodedValue::new(None)
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

    fn encoded_value(&self) -> EncodedValue {
        EncodedValue::new(None)
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

    fn encoded_value(&self) -> EncodedValue {
        EncodedValue::new(None)
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

    fn encoded_value(&self) -> EncodedValue {
        EncodedValue::new(None)
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

    fn encoded_value(&self) -> EncodedValue {
        EncodedValue::new(None)
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

    fn encoded_value(&self) -> EncodedValue {
        EncodedValue::new(None)
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

    fn encoded_value(&self) -> EncodedValue {
        EncodedValue::new(None)
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

    fn encoded_value(&self) -> EncodedValue {
        EncodedValue::new(None)
    }
}

unsafe impl<'a, T> OdbcType<'a> for Option<T>
where
    T: OdbcType<'a>,
{
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

    fn null_bytes_count() -> usize {
        T::null_bytes_count()
    }

    fn encoded_value(&self) -> EncodedValue {
        EncodedValue::new(None)
    }
}

mod test {
    // use environment::create_environment_v3_with_os_db_encoding;

    #[test]
    fn encoded_value_test() {
        use OdbcType;

        let mut checker = std::collections::HashSet::new();
        let mut encoded_values = Vec::new();

        // let _ = create_environment_v3_with_os_db_encoding("utf8", "sjis");

        //string test
        for i in 0..10 {
            for h in 0..10 {
                let string_value = format!("{}{}", i, h);
                // println!("org value => {}    address => {:?}", string_value, string_value.value_ptr());

                let enc = string_value.encoded_value();
                // println!("{} {:?}", enc.column_size(), enc.buf);
                if checker.len() == 0 || !checker.contains(&enc.value_ptr()) {
                    checker.insert(enc.value_ptr());
                    encoded_values.push(enc);
                } else {
                    panic!("same address occur!");
                }
            }
        }
        checker.clear();
        encoded_values.clear();

        //&str test
        for i in 0..10 {
            for h in 0..10 {
                let str_value: &str = &format!("{}{}", i, h);
                // println!("org value => {}    address => {:?}", str_value, str_value.value_ptr());

                let enc = str_value.encoded_value();
                if checker.len() == 0 || !checker.contains(&enc.value_ptr()) {
                    checker.insert(enc.value_ptr());
                    encoded_values.push(enc);
                } else {
                    panic!("same address occur!");
                }
            }
        }
        checker.clear();
        encoded_values.clear();

        //Cow<str> test
        for i in 0..10 {
            for h in 0..10 {
                let cow_value: std::borrow::Cow<str> =
                    std::borrow::Cow::from(format!("{}{}", i, h));
                // println!("org value => {}    address => {:?}", cow_value, cow_value.value_ptr());

                let enc = cow_value.encoded_value();
                if checker.len() == 0 || !checker.contains(&enc.value_ptr()) {
                    checker.insert(enc.value_ptr());
                    encoded_values.push(enc);
                } else {
                    panic!("same address occur!");
                }
            }
        }
        checker.clear();
        encoded_values.clear();
    }
}
