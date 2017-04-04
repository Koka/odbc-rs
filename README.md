# ODBC wrapper for safe idiomatic Rust

Library for writing [ODBC](https://msdn.microsoft.com/en-us/library/ms710154.aspx) applications in Rust.

[![https://travis-ci.org/Koka/odbc-rs](https://travis-ci.org/Koka/odbc-rs.svg?branch=master)](https://travis-ci.org/Koka/odbc-rs)
[![Build status](https://ci.appveyor.com/api/projects/status/45ovhoic0wg7mnv5/branch/master?svg=true)](https://ci.appveyor.com/project/Koka/odbc-rs/branch/master)
[![https://crates.io/crates/odbc](https://meritbadge.herokuapp.com/odbc#nocache6)](https://crates.io/crates/odbc)
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

    let env = Environment::new().unwrap();
    let env3 = env.set_odbc_version_3()?;

    let conn = DataSource::with_parent(&env3)?;

    let mut buffer = String::new();
    println!("Please enter connection string: ");
    io::stdin().read_line(&mut buffer).unwrap();

    let mut conn = conn.connect_with_connection_string(&buffer)?;
    execute_statement(&mut conn)
}

fn execute_statement(mut conn: &mut DataSource<Connected>) -> Result<()> {
    let stmt = Statement::with_parent(&mut conn)?;

    let mut sql_text = String::new();
    println!("Please enter SQL statement string: ");
    io::stdin().read_line(&mut sql_text).unwrap();

    match stmt.exec_direct(&sql_text)?{
        Data(mut stmt) =>{
            let cols = stmt.num_result_cols()?;
            while let Some(mut cursor) = stmt.fetch()? {
                for i in 1..(cols + 1) {
                    match cursor.get_data::<String>(i as u16)? {
                        Some(val) => print!(" {}", val),
                        None => print!(" NULL"),
                    }
                }
                println!("");
            }
        }
        NoData(_) => println!("Query executed, no data returned")
    }

    Ok(())
}
```
