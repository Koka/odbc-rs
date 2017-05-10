extern crate odbc;
// Use this crate and set environment variable RUST_LOG=odbc to see ODBC warnings
extern crate env_logger;

use odbc::*;

fn main() {
    env_logger::init().unwrap();
    test_me().unwrap()
}

fn test_me() -> std::result::Result<(), DiagnosticRecord> {
    let env = Environment::new().expect("Can't create ODBC environment").set_odbc_version_3()?;
    let conn = DataSource::with_parent(&env)?.connect("PostgreSQL", "postgres", "postgres")?;
    let result = Statement::with_parent(&conn)?.exec_direct("select '1' as str, 1 as num, current_timestamp as timestamp, null as nul, true as boolean")?;

    if let Data(stmt) = result {
        for i in 1..5 {
            let val = stmt.describe_col(i)?;
            println!("Column {}: {:?}", i, val)
        }
    };

    Ok(())
}
