# Rust ODBC FFI binding

Library for writing [ODBC](https://msdn.microsoft.com/en-us/library/ms710154.aspx) applications in Rust.

[![https://travis-ci.org/Koka/odbc-rs](https://travis-ci.org/Koka/odbc-rs.svg?branch=master)](https://travis-ci.org/Koka/odbc-rs)
[![https://crates.io/crates/odbc](https://meritbadge.herokuapp.com/odbc#nocache2)](https://crates.io/crates/odbc)
[![Coverage Status](https://coveralls.io/repos/github/Koka/odbc-rs/badge.svg)](https://coveralls.io/github/Koka/odbc-rs)

Docs are available [here](http://koka.github.io/odbc-rs/odbc/)

```rust
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

    let mut env = Environment::new().unwrap();
    env.set_odbc_version_3()?;

    let mut conn = DataSource::with_parent(&env)?;

    let mut buffer = String::new();
    println!("Please enter connection string: ");
    io::stdin().read_line(&mut buffer).unwrap();

    conn.connect_with_connection_string(&buffer)?;
    execute_statement(&mut conn)?;
    conn.disconnect()?;
    Ok(())
}

//Execute statement in smaller scope, so it gets deallocated before disconnect
fn execute_statement(mut conn: &mut DataSource) -> Result<()> {
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
```