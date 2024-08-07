//! Contains test for all types supporting `get_data`
//!
//! These tests assume there is a Stage table with a Varchar in 'A', an Integer in 'B' and a Real
//! in 'C'
extern crate odbc;
use odbc::*;
use std::ffi::CString;

const A: &'static str = "SELECT A FROM TEST_TYPES;";
const B: &'static str = "SELECT B FROM TEST_TYPES;";
const C: &'static str = "SELECT C FROM TEST_TYPES;";

macro_rules! test_type {
    ($t:ty, $c:expr, $e:expr) => {{
        let env = create_environment_v3().unwrap();
        let conn = env.connect("TestDataSource", "", "").unwrap();
        let stmt = Statement::with_parent(&conn).unwrap();
        if let Ok(Data(mut cursor)) = stmt.exec_direct($c) {
            if let Ok(Some(mut row)) = cursor.fetch() {
                let value: $t = row.get_data(1).unwrap().unwrap();
                assert_eq!(value, $e);
            } else {
                panic!("Result set has been empty");
            }
        } else {
            panic!("SELECT did not return result set");
        };
    }};
}

#[test]
fn _ref_str() {
    test_type!(&str, A, "Hello, World!")
}

#[test]
fn _string() {
    test_type!(String, A, String::from("Hello, World!"))
}

#[test]
fn _c_string() {
    let buf = "Hello, World!".as_bytes();
    test_type!(CString, A, CString::new(buf).unwrap())
}

#[test]
fn _slice() {
    let buf = "Hello, World!".as_bytes();
    test_type!(&[u8], A, buf)
}

#[test]
fn _vec() {
    let buf = "Hello, World!".as_bytes().to_vec();
    test_type!(Vec<u8>, A, buf)
}

#[test]
fn _i8() {
    test_type!(i8, B, 42)
}

#[test]
fn _u8() {
    test_type!(u8, B, 42)
}

#[test]
fn _i16() {
    test_type!(i16, B, 42)
}

#[test]
fn _u16() {
    test_type!(u16, B, 42)
}

#[test]
fn _i32() {
    test_type!(i32, B, 42)
}

#[test]
fn _u32() {
    test_type!(u32, B, 42)
}

#[test]
fn _i64() {
    test_type!(i64, B, 42)
}

#[test]
fn _u64() {
    test_type!(u64, B, 42)
}

#[test]
fn _f32() {
    test_type!(f32, C, 3.14)
}

#[test]
fn _f64() {
    test_type!(f64, C, 3.14)
}
