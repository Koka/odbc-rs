extern crate odbc;
// Use this crate and set environment variable RUST_LOG=odbc to see ODBC warnings
extern crate env_logger;

use odbc::*;

fn main() {
    env_logger::init();
    test_me().unwrap()
}

fn test_me() -> std::result::Result<(), DiagnosticRecord> {
    let env = create_environment_v3().expect("Can't create ODBC environment");
    let conn = env.connect("PostgreSQL", "postgres", "postgres")?;
    let stmt = Statement::with_parent(&conn)?.prepare("select version() where ? = ?")?;

    let param = "FOOBAR";

    let stmt = stmt.bind_parameter(1, &param)?;
    let stmt = stmt.bind_parameter(2, &param)?;
    let stmt = stmt.bind_parameter(3, &param)?;
    let stmt = stmt.bind_parameter(4, &param)?;

    let stmt = if let Data(mut stmt) = stmt.execute()? {
        if let Some(mut cursor) = stmt.fetch()? {
            println!("{}", cursor.get_data::<String>(1)?.unwrap());
        }
        stmt.close_cursor()?
    } else {
        panic!("SELECT statement returned no result set");
    };

    let stmt = stmt.reset_parameters()?;

    let param = 128u8;

    let stmt = stmt.bind_parameter(1, &param)?;
    let stmt = stmt.bind_parameter(2, &param)?;

    let stmt = if let Data(mut stmt) = stmt.execute()? {
        if let Some(mut cursor) = stmt.fetch()? {
            println!("{}", cursor.get_data::<String>(1)?.unwrap());
        }
        stmt.close_cursor()?
    } else {
        panic!("SELECT statement returned no result set");
    };
    stmt.reset_parameters().unwrap();

    Ok(())
}
