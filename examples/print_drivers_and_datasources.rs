// Use this crate and set environmet variable RUST_LOG=odbc to see ODBC warnings
extern crate env_logger;
extern crate odbc;
use odbc::*;

fn main() {

    match print_drivers_and_datasources() {
        Ok(()) => (),
        Err(err) => println!("{}", err),
    }
}

fn print_drivers_and_datasources() -> odbc::Result<()> {

    env_logger::init().unwrap();

    let env = Environment::new().unwrap();
    let mut env3 = env.set_odbc_version_3()?;

    println!("Driver list:");
    for driver_info in env3.drivers()? {
        println!("\nDriver Name: {}", driver_info.description);
        for (key, value) in driver_info.attributes {
            println!("    {}={}", key, value);
        }
    }

    println!("\nDataSource list:");
    for ds in env3.data_sources()? {
        println!("    {}\n    {}\n\n", ds.server_name, ds.driver);
    }
    Ok(())
}
