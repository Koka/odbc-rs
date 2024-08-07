//! Contains test for all types supporting `bind_parameter`
//!
//! These tests assume there is a Stage table with a Varchar in 'A', an Integer in 'B' and a Real
//! in 'C'
extern crate odbc;
use odbc::*;
use std::ffi::CString;

const A: &'static str = "SELECT A FROM TEST_TYPES WHERE A = ?;";
const B: &'static str = "SELECT B FROM TEST_TYPES WHERE B = ?;";
const C: &'static str = "SELECT C FROM TEST_TYPES WHERE C = ?;";

macro_rules! test_type {
    ($c:expr, $e:expr) => {{
        let env = create_environment_v3().unwrap();
        let conn = env.connect("TestDataSource", "", "").unwrap();
        let stmt = Statement::with_parent(&conn).unwrap();
        let stmt = stmt.bind_parameter(1, $e).unwrap();
        if let Ok(Data(mut stmt)) = stmt.exec_direct($c) {
            if let Some(_) = stmt.fetch().unwrap() {
                //DO NOTHING
            } else {
                panic!("Result set has been empty");
            }
        } else {
            panic!("SELECT did not return result set");
        };
    }};
}

#[test]
#[ignore]
fn _slice() {
    //TODO: ignored due to weird SQLite VARBINARY handling, may be it works in other DBs
    let param = "Hello, World!".as_bytes();
    test_type!(A, &param)
}

#[test]
#[ignore]
fn _vec() {
    //TODO: ignored due to weird SQLite VARBINARY handling, may be it works in other DBs
    let param = "Hello, World!".as_bytes().to_vec();
    test_type!(A, &param)
}

#[test]
fn _ref_str() {
    let param = "Hello, World!";
    test_type!(A, &param)
}

#[test]
fn _string() {
    let param = String::from("Hello, World!");
    test_type!(A, &param)
}

#[test]
fn _c_string() {
    let buf = "Hello, World!".as_bytes();
    let param = CString::new(buf).unwrap();
    test_type!(A, &param)
}

#[test]
fn _i8() {
    let param: i8 = 42;
    test_type!(B, &param)
}

#[test]
fn _u8() {
    let param: u8 = 42;
    test_type!(B, &param)
}

#[test]
fn _i16() {
    let param: i16 = 42;
    test_type!(B, &param)
}

#[test]
fn _u16() {
    let param: u16 = 42;
    test_type!(B, &param)
}

#[test]
fn _i32() {
    let param: i32 = 42;
    test_type!(B, &param)
}

#[test]
fn _u32() {
    let param: u32 = 42;
    test_type!(B, &param)
}

#[test]
fn _i64() {
    let param: i64 = 42;
    test_type!(B, &param)
}

#[test]
fn _u64() {
    let param: u64 = 42;
    test_type!(B, &param)
}

#[test]
fn _f32() {
    let param: f32 = 42.0;
    test_type!(B, &param)
}

#[test]
fn _f64() {
    let param: f64 = 3.14;
    test_type!(C, &param)
}
