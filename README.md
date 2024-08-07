# ODBC wrapper for safe idiomatic Rust

Library for writing [ODBC](https://msdn.microsoft.com/en-us/library/ms710154.aspx) applications in Rust.

If you're looking for raw ODBC FFI bindings check [odbc-safe](https://github.com/pacman82/odbc-safe) and [odbc-sys](https://github.com/pacman82/odbc-sys) crate.

[![Project Status: Unsupported – The project has reached a stable, usable state but the author(s) have ceased all work on it. A new maintainer may be desired.](https://www.repostatus.org/badges/latest/unsupported.svg)](https://www.repostatus.org/#unsupported)

[![https://travis-ci.org/Koka/odbc-rs](https://travis-ci.org/Koka/odbc-rs.svg?branch=master)](https://travis-ci.org/Koka/odbc-rs)
[![Build status](https://ci.appveyor.com/api/projects/status/45ovhoic0wg7mnv5/branch/master?svg=true)](https://ci.appveyor.com/project/Koka/odbc-rs/branch/master)
[![https://crates.io/crates/odbc](https://meritbadge.herokuapp.com/odbc#nocache8)](https://crates.io/crates/odbc)
[![Coverage Status](https://coveralls.io/repos/github/Koka/odbc-rs/badge.svg)](https://coveralls.io/github/Koka/odbc-rs)
[![Docs](https://docs.rs/odbc/badge.svg)](https://docs.rs/odbc)
[![Join the chat at https://gitter.im/odbc-rs/odbc](https://badges.gitter.im/Join%20Chat.svg)](https://gitter.im/odbc-rs/odbc)

Docs are also available [here](http://koka.github.io/odbc-rs/odbc/)

```rust
extern crate odbc;
// Use this crate and set environment variable RUST_LOG=odbc to see ODBC warnings
extern crate env_logger;
use odbc::*;
use std::io;
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

    let mut buffer = String::new();
    println!("Please enter connection string: ");
    io::stdin().read_line(&mut buffer).unwrap();

    let conn = env.connect_with_connection_string(&buffer)?;
    execute_statement(&conn)
}

fn execute_statement<'env>(conn: &Connection<'env, AutocommitOn>) -> Result<()> {
    let stmt = Statement::with_parent(conn)?;

    let mut sql_text = String::new();
    println!("Please enter SQL statement string: ");
    io::stdin().read_line(&mut sql_text).unwrap();

    match stmt.exec_direct(&sql_text)? {
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

    Ok(())
}

```
