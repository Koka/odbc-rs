//! Contains test for all types supporting `bind_parameter`
//!
//! These tests assume there is a Stage table with a Varchar in 'A', an Integer in 'B' and a Real
//! in 'C'
extern crate odbc;
use odbc::*;

#[test]
#[ignore]
/// tested in windows中文版 codepage 936
fn _exec_direct() {
    let env = create_environment_v3().unwrap();
    let conn = env.connect("TestDataSource", "", "").unwrap();
    let stmt = Statement::with_parent(&conn).unwrap();

    // select '你好' as hello
    if let Ok(Data(mut stmt)) = stmt.exec_direct_bytes(
        vec![
            115, 101, 108, 101, 99, 116, 32, 39, 196, 227, 186, 195, 39, 32, 97, 115, 32, 104, 101,
            108, 108, 111, 32,
        ]
        .as_slice(),
    ) {
        while let Some(mut cursor) = stmt.fetch().unwrap() {
            match cursor.get_data::<Vec<u8>>(1).unwrap() {
                Some(val) => assert_eq!(val, vec![196, 227, 186, 195]), // when  你好 is encoded by gbk, it is [196, 227, 186, 195]
                None => panic!(" NULL"),
            }
        }
    } else {
        panic!("SELECT did not return result set");
    };
}

#[test]
#[ignore]
/// tested in windows中文版 codepage 936
fn _prepare_1() {
    let env = create_environment_v3().unwrap();
    let conn = env.connect("TestDataSource", "", "").unwrap();
    // select '你好' as hello where 1 = ?
    let stmt = Statement::with_parent(&conn)
        .unwrap()
        .prepare_bytes(
            vec![
                115, 101, 108, 101, 99, 116, 32, 39, 196, 227, 186, 195, 39, 32, 97, 115, 32, 104,
                101, 108, 108, 111, 32, 119, 104, 101, 114, 101, 32, 32, 49, 32, 61, 32, 63, 32,
            ]
            .as_slice(),
        )
        .unwrap();
    let stmt = stmt.bind_parameter(1, &1).unwrap();

    match stmt.execute().unwrap() {
        Data(mut stmt) => {
            while let Some(mut cursor) = stmt.fetch().unwrap() {
                match cursor.get_data::<Vec<u8>>(1).unwrap() {
                    Some(val) => assert_eq!(val, vec![196, 227, 186, 195]),
                    None => panic!(" NULL"),
                }
            }
        }
        NoData(_) => {
            panic!("SELECT did not return result set");
        }
    };
}

#[test]
#[ignore]
/// tested in windows中文版 codepage 936
fn _prepare_2() {
    let env = create_environment_v3().unwrap();
    let conn = env.connect("TestDataSource", "sa", "123456").unwrap();
    // select '你好' as hello where '你好' = ?
    let stmt = Statement::with_parent(&conn)
        .unwrap()
        .prepare_bytes(
            vec![
                115, 101, 108, 101, 99, 116, 32, 39, 228, 189, 160, 229, 165, 189, 39, 32, 97, 115,
                32, 104, 101, 108, 108, 111, 32, 119, 104, 101, 114, 101, 32, 39, 228, 189, 160,
                229, 165, 189, 39, 32, 61, 32, 63,
            ]
            .as_slice(),
        )
        .unwrap();
    // bind gbk encoded byte
    let param = CustomOdbcType {
        data: &[228, 189, 160, 229, 165, 189],
    };
    let stmt = stmt.bind_parameter(1, &param).unwrap();

    if let Ok(Data(mut stmt)) = stmt.execute() {
        if let Some(mut cursor) = stmt.fetch().unwrap() {
            match cursor.get_data::<Vec<u8>>(1).unwrap() {
                Some(val) => assert_eq!(val, vec![196, 227, 186, 195]),
                None => panic!(" NULL"),
            }
        } else {
            panic!("No data")
        }
    } else {
        panic!("SELECT did not return result set");
    };
}

#[test]
fn exec_direct() {
    let env = create_environment_v3().unwrap();
    let conn = env.connect("TestDataSource", "", "").unwrap();
    let stmt = Statement::with_parent(&conn).unwrap();

    // select 'hello'
    if let Ok(Data(mut stmt)) = stmt.exec_direct_bytes(
        vec![
            115, 101, 108, 101, 99, 116, 32, 39, 104, 101, 108, 108, 111, 39, 32,
        ]
        .as_slice(),
    ) {
        if let Some(mut cursor) = stmt.fetch().unwrap() {
            match cursor.get_data::<Vec<u8>>(1).unwrap() {
                Some(val) => assert_eq!(val, vec![104, 101, 108, 108, 111]), // when  hello is encoded by utf8, it is [104, 101, 108, 108, 111]
                None => panic!(" NULL"),
            }
        } else {
            panic!("No Data");
        }
    } else {
        panic!("SELECT did not return result set");
    };
}

#[test]
fn prepare_1() {
    let env = create_environment_v3().unwrap();
    let conn = env.connect("TestDataSource", "", "").unwrap();
    // select 'hello' where 'hello' = ?
    let stmt = Statement::with_parent(&conn)
        .unwrap()
        .prepare_bytes(
            vec![
                115, 101, 108, 101, 99, 116, 32, 39, 104, 101, 108, 108, 111, 39, 32, 119, 104,
                101, 114, 101, 32, 39, 104, 101, 108, 108, 111, 39, 32, 61, 32, 63, 32,
            ]
            .as_slice(),
        )
        .unwrap();
    let stmt = stmt.bind_parameter(1, &"hello").unwrap();

    if let Ok(Data(mut stmt)) = stmt.execute() {
        if let Some(mut cursor) = stmt.fetch().unwrap() {
            match cursor.get_data::<Vec<u8>>(1).unwrap() {
                Some(val) => assert_eq!(val, vec![104, 101, 108, 108, 111]),
                None => panic!(" NULL"),
            }
        } else {
            panic!("No data");
        }
    } else {
        panic!("SELECT did not return result set");
    };
}

/// CustomOdbcType  for bindParameter
struct CustomOdbcType<'a> {
    data: &'a [u8],
}

unsafe impl<'a> OdbcType<'a> for CustomOdbcType<'a> {
    fn sql_data_type() -> ffi::SqlDataType {
        ffi::SQL_VARCHAR
    }
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_CHAR
    }

    fn convert(buffer: &'a [u8]) -> Self {
        CustomOdbcType { data: buffer }
    }

    fn column_size(&self) -> ffi::SQLULEN {
        self.data.len() as ffi::SQLULEN
    }

    fn value_ptr(&self) -> ffi::SQLPOINTER {
        self.data.as_ptr() as *const Self as ffi::SQLPOINTER
    }

    fn encoded_value(&self) -> EncodedValue {
        EncodedValue::new(None)
    }
}

#[test]
fn prepare_2() {
    let env = create_environment_v3().unwrap();
    let conn = env.connect("TestDataSource", "", "").unwrap();
    // select 'hello' where 'hello' = ?
    let stmt = Statement::with_parent(&conn)
        .unwrap()
        .prepare_bytes(
            vec![
                115, 101, 108, 101, 99, 116, 32, 39, 104, 101, 108, 108, 111, 39, 32, 119, 104,
                101, 114, 101, 32, 39, 104, 101, 108, 108, 111, 39, 32, 61, 32, 63, 32,
            ]
            .as_slice(),
        )
        .unwrap();
    // bind utf encoded byte
    // let param: Vec<u8> = vec![104, 101, 108, 108, 111];
    let param = CustomOdbcType {
        data: &[104, 101, 108, 108, 111],
    };
    let stmt = stmt.bind_parameter(1, &param).unwrap();

    if let Ok(Data(mut stmt)) = stmt.execute() {
        if let Some(mut cursor) = stmt.fetch().unwrap() {
            match cursor.get_data::<Vec<u8>>(1).unwrap() {
                Some(val) => assert_eq!(val, vec![104, 101, 108, 108, 111]),
                None => panic!(" NULL"),
            }
        } else {
            panic!("No data");
        }
    } else {
        panic!("SELECT did not return result set");
    };
}
