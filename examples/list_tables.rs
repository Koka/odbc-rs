extern crate odbc;
// Use this crate and set environmet variable RUST_LOG=odbc to see ODBC warnings
extern crate env_logger;
extern crate odbc_safe;

use odbc::*;
use odbc_safe::AutocommitOn;

fn main() {
    env_logger::init();

    match connect() {
        Ok(()) => println!("Success"),
        Err(diag) => println!("Error: {}", diag),
    }
}

fn connect() -> std::result::Result<(), DiagnosticRecord> {
    let env = create_environment_v3().map_err(|e| e.unwrap())?;
    let conn = env.connect("TestDataSource", "", "").unwrap();
    list_tables(&conn)
}

fn list_tables(conn: &Connection<AutocommitOn>) -> Result<()> {
    let stmt = Statement::with_parent(conn)?;
    let mut rs = stmt.tables_str("%", "%", "%", "TABLE")?;
    let cols = rs.num_result_cols()?;
    while let Some(mut cursor) = rs.fetch()? {
        for i in 1..(cols + 1) {
            match cursor.get_data::<&str>(i as u16)? {
                Some(val) => print!(" {}", val),
                None => print!(" NULL"),
            }
        }
        println!();
    }

    Ok(())
}
