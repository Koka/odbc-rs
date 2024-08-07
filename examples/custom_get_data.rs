extern crate odbc;
// Use this crate and set environment variable RUST_LOG=odbc to see ODBC warnings
extern crate chrono;
extern crate env_logger;
extern crate odbc_safe;

use chrono::prelude::*;
use odbc::*;
use odbc_safe::AutocommitMode;

trait Extract {
    fn extract<T>(&mut self, index: u16) -> Option<T>
    where
        T: MySupportedType;
}

impl<'s, 'a: 's, S: 's, AC: AutocommitMode> Extract for Cursor<'s, 'a, 'a, S, AC> {
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
    fn extract_from<'a, 'con, S, AC: AutocommitMode>(
        cursor: &mut odbc::Cursor<'a, 'con, 'con, S, AC>,
        index: u16,
    ) -> Option<Self>;
}

impl MySupportedType for DateTime<Local> {
    fn extract_from<'a, 'con, S, AC: AutocommitMode>(
        cursor: &mut odbc::Cursor<'a, 'con, 'con, S, AC>,
        index: u16,
    ) -> Option<Self> {
        cursor
            .get_data(index)
            .expect("Can't get column")
            .map(|datetime: String| {
                NaiveDateTime::parse_from_str(&datetime, "%Y-%m-%d %H:%M:%S%.f")
                    .map(|ndt| Local.from_local_datetime(&ndt))
                    .unwrap()
                    .single()
                    .unwrap()
            })
    }
}

fn main() {
    env_logger::init();
    println!("Success: {}", test_me().unwrap().expect("No result!"))
}

fn test_me() -> std::result::Result<Option<DateTime<Local>>, DiagnosticRecord> {
    let env = create_environment_v3().map_err(|e| e.expect("Can't create ODBC environment"))?;
    let conn = env.connect("PostgreSQL", "postgres", "postgres")?;
    let mut result = Statement::with_parent(&conn)?.exec_direct("select current_timestamp")?;

    if let &mut Data(ref mut stmt) = &mut result {
        let val = stmt
            .fetch()
            .expect("Can't get cursor")
            .and_then(|mut cursor| cursor.extract(1));
        Ok(val)
    } else {
        Ok(None)
    }
}
