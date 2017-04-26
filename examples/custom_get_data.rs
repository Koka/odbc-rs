extern crate odbc;
// Use this crate and set environment variable RUST_LOG=odbc to see ODBC warnings
extern crate env_logger;
extern crate chrono;

use std::fmt;
use odbc::*;
use chrono::prelude::*;

// We need to define own type here as rust won't allow to implement remote trait for remote type, so we make a type local for this crate.
struct MyDateTime(DateTime<Local>);

impl fmt::Display for MyDateTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MyDateTime {}", self.0)
    }
}

// Let's define how to get data from result set for our custom type
unsafe impl<'a> Output<'a> for MyDateTime {
    fn get_data(stmt: &mut Raii<ffi::Stmt>,
                col_or_param_num: u16,
                buffer: &'a mut [u8])
                -> Return<Option<Self>> {
        unsafe {
            let mut indicator: ffi::SQLLEN = 0;

            //Let's ask ODBC driver to provide us with SQL_C_CHAR representation and then parse it to our custom type
            let result = ffi::SQLGetData(stmt.handle(),
                                         col_or_param_num,
                                         ffi::SQL_C_CHAR,
                                         buffer.as_mut_ptr() as ffi::SQLPOINTER,
                                         buffer.len() as ffi::SQLLEN,
                                         &mut indicator as *mut ffi::SQLLEN);

            match result {
                ffi::SQL_SUCCESS => {
                    if indicator == ffi::SQL_NULL_DATA {
                        Return::Success(None)
                    } else {
                        let str = std::str::from_utf8(&buffer[..(indicator as usize)]).unwrap();
                        let dt = Local.datetime_from_str(str, "%Y-%m-%d %H:%M:%S%.f").unwrap();
                        Return::Success(Some(MyDateTime(dt)))
                    }
                }
                ffi::SQL_SUCCESS_WITH_INFO => {
                    if indicator == ffi::SQL_NULL_DATA {
                        Return::Success(None)
                    } else {
                        let str = std::str::from_utf8(&buffer[..(indicator as usize)]).unwrap();
                        let dt = Local.datetime_from_str(str, "%Y-%m-%d %H:%M:%S%.f").unwrap();
                        Return::Success(Some(MyDateTime(dt)))
                    }
                }
                ffi::SQL_ERROR => Return::Error,
                ffi::SQL_NO_DATA => panic!("SQLGetData has already returned the column data"),
                r => panic!("unexpected return value from SQLGetData: {:?}", r),
            }
        }
    }
}

fn main() {
    env_logger::init().unwrap();
    println!("Success: {}", test_me().unwrap().expect("No result!"))
}

fn test_me() -> std::result::Result<Option<MyDateTime>, DiagnosticRecord> {
    let env = Environment::new().expect("Can't create ODBC environment").set_odbc_version_3()?;
    let conn = DataSource::with_parent(&env)?.connect("PostgreSQL", "postgres", "postgres")?;
    let result = Statement::with_parent(&conn)?.exec_direct("select current_timestamp")?;

    let mut val = None;

    if let Data(mut stmt) = result {
        val = stmt.fetch()
            .expect("Can't get cursor")
            .and_then(|mut cursor| {
                cursor.get_data::<MyDateTime>(1)
                    .expect("Can't get column")
            })
    };

    Ok(val)
}
