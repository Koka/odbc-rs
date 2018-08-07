// Use this crate and set environment variable RUST_LOG=odbc to see ODBC warnings
extern crate env_logger;
extern crate odbc;

use std::error::Error;
use odbc::*;

fn main() -> Result<(), Box<Error>> {
    env_logger::init();

    println!("Driver list:");
    for driver_info in Env::drivers()? {
        println!("\nDriver Name: {}", driver_info.description);
        for (key, value) in driver_info.attributes {
            println!("\t{} = {}", key, value);
        }
    }

    println!("\nDataSource list:");
    for ds in Env::data_sources()? {
        println!("\n\t{}: {}", ds.server_name, ds.driver);
    }

    Ok(())
}
