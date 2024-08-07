extern crate odbc;
// Use this crate and set environmet variable RUST_LOG=odbc to see ODBC warnings
extern crate env_logger;
extern crate odbc_safe;

use odbc::*;
use odbc_safe::AutocommitOff;

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
    let mut conn = conn.disable_autocommit().unwrap();
    list_tables(&mut conn)
}

fn list_tables(conn: &mut Connection<AutocommitOff>) -> Result<()> {
    let stmt = Statement::with_parent(conn)?;
    match stmt.exec_direct("SELECT 'HELLO' FROM MOVIES")? {
        Data(mut stmt) => {
            let cols = stmt.num_result_cols()?;
            while let Some(mut cursor) = stmt.fetch()? {
                for i in 1..(cols + 1) {
                    match cursor.get_data::<&str>(i as u16)? {
                        Some(val) => print!(" {}", val),
                        None => print!(" NULL"),
                    }
                }
                println!("");
            }
        }
        NoData(_) => println!("Query executed, no data returned"),
    }

    conn.commit()
}
