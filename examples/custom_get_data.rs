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

impl <'a> OdbcType<'a> for MyDateTime {
    fn c_data_type() -> ffi::SqlCDataType {
        ffi::SQL_C_CHAR
    }

    fn convert(buffer: &'a [u8]) -> Self {
        let str = std::str::from_utf8(buffer).unwrap();
        let dt = Local.datetime_from_str(str, "%Y-%m-%d %H:%M:%S%.f").unwrap();
        MyDateTime(dt)
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
