extern crate odbc;
// Use this crate and set environmet variable RUST_LOG=odbc to see ODBC warnings
extern crate env_logger;
use odbc::*;
use std::io;

fn main() {

    env_logger::init().unwrap();

    match connect() {
        Ok(()) => println!("Success"),
        Err(diag) => println!("Error: {}", diag),
    }
}

fn connect() -> std::result::Result<(), DiagnosticRecord> {

    let env = Environment::new().unwrap();
    let env3 = env.set_odbc_version_3()?;

    let conn = DataSource::with_parent(&env3)?;

    let mut buffer = String::new();
    println!("Please enter connection string: ");
    io::stdin().read_line(&mut buffer).unwrap();

    let mut conn = conn.connect_with_connection_string(&buffer)?;
    execute_statement(&mut conn)?;
    conn.disconnect()?;
    Ok(())
}

//Execute statement in smaller scope, so it gets deallocated before disconnect
fn execute_statement(mut conn: &mut DataSource<Connected>) -> Result<()> {
    //Execute statement in smaller scope, so it gets deallocated before disconnect
    let mut stmt = Statement::with_parent(&mut conn)?;

    let mut sql_text = String::new();
    println!("Please enter SQL statement string: ");
    io::stdin().read_line(&mut sql_text).unwrap();

    assert!(stmt.exec_direct(&sql_text)?);
    let cols = stmt.num_result_cols()?;

    while stmt.fetch()? {
        for i in 1..(cols + 1) {
            match stmt.get_data(i as u16)? {
                Some(val) => print!(" {}", val),
                None => print!(" NULL"),
            }
        }
        println!("");
    }
    Ok(())
}

