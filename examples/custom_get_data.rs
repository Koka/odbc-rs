extern crate odbc;
// Use this crate and set environment variable RUST_LOG=odbc to see ODBC warnings
extern crate env_logger;
extern crate chrono;

use odbc::*;
use chrono::prelude::*;

trait Extract {
    fn extract<T>(&mut self, index: u16) -> Option<T>
    where
        T: MySupportedType;
}

impl<'a, S> Extract for Cursor<'a, 'a, 'a, S> {
    fn extract<T>(&mut self, index: u16) -> Option<T>
    where
        T: MySupportedType,
    {
        MySupportedType::extract_from(self, index)
    }
}

trait MySupportedType
where
    Self: Sized,
{
    fn extract_from<'a, S>(cursor: &mut odbc::Cursor<'a, 'a, 'a, S>, index: u16) -> Option<Self>;
}

impl MySupportedType for DateTime<Local> {
    fn extract_from<'a, S>(cursor: &mut odbc::Cursor<'a, 'a, 'a, S>, index: u16) -> Option<Self> {
        cursor.get_data(index).expect("Can't get column").map(
            |datetime: String| {
                Local
                    .datetime_from_str(&datetime, "%Y-%m-%d %H:%M:%S%.f")
                    .unwrap()
            },
        )
    }
}

fn main() {
    env_logger::init().unwrap();
    println!("Success: {}", test_me().unwrap().expect("No result!"))
}

fn test_me() -> std::result::Result<Option<DateTime<Local>>, DiagnosticRecord> {
    let env = create_environment_v3().map_err(|e| {
        e.expect("Can't create ODBC environment")
    })?;
    let conn = DataSource::with_parent(&env)?.connect(
        "PostgreSQL",
        "postgres",
        "postgres",
    )?;
    let result = Statement::with_parent(&conn)?.exec_direct(
        "select current_timestamp",
    )?;

    let mut val = None;

    if let Data(mut stmt) = result {
        val = stmt.fetch().expect("Can't get cursor").and_then(
            |mut cursor| {
                cursor.extract(1)
            },
        )
    };

    Ok(val)
}
