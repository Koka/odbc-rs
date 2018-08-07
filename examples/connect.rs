// Use this crate and set environment variable RUST_LOG=odbc to see ODBC warnings
extern crate env_logger;
extern crate odbc;

use std::error::Error;
use odbc::*;
use std::io;
use std::str::from_utf8;

fn main() -> Result<(), Box<Error>> {
    env_logger::init();

    let mut sql_text = String::new();
    println!("Please enter SQL statement string: ");
    io::stdin().read_line(&mut sql_text)?;

    Env::with_connection("DSN=TestDataSource", |conn| {
        conn.select(sql_text, |result_set| {
            let mut result_set = result_set.expect("No result set!");
            let description = result_set.describe_columns()?;

            println!("Columns:");
            for col in description {
                print!("{} {:?} {:?}\t", from_utf8(&col.name).unwrap_or("Non UTF-8 string"), col.data_type, col.nullable)
            }
            println!();

            println!("Result:");
            for mut row in result_set.rows()? {
                for col in 1..(row.length()) {
                    print!("{:?}\t", row.get_col(col));
                }
                println!();
            }

            Ok(())
        })
    })
}
