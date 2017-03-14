extern crate odbc;
use odbc::*;

#[test]
fn list_tables() {

    let env = Environment::new().unwrap();
    let env = env.set_odbc_version_3().unwrap();
    let ds = DataSource::with_parent(&env).unwrap();
    let mut ds = ds.connect("PostgreSQL", "postgres", "").unwrap();
    // scope is required (for now) to close statement before disconnecting
    {
        let mut statement = Statement::with_parent(&mut ds).unwrap();
        statement.tables().unwrap();
        assert_eq!(statement.num_result_cols().unwrap(), 5);
    }
    ds.disconnect().unwrap();
}

#[test]
fn not_read_only() {

    let env = Environment::new().unwrap();
    let env = env.set_odbc_version_3().unwrap();
    let conn = DataSource::with_parent(&env).unwrap();
    let conn = conn.connect("PostgreSQL", "postgres", "").unwrap();

    assert!(!conn.read_only().unwrap());
    conn.disconnect().unwrap();
}

#[test]
fn implicit_disconnect() {

    let env = Environment::new().unwrap();
    let env = env.set_odbc_version_3().unwrap();
    let conn = DataSource::with_parent(&env).unwrap();
    conn.connect("PostgreSQL", "postgres", "").unwrap();

    // if there would be no implicit disconnect, all the drops would panic with function sequence
    // error
}

#[test]
fn invalid_connection_string() {

    let expected = if cfg!(target_os = "windows") {
        "State: IM002, Native error: 0, Message: [Microsoft][ODBC Driver Manager] Data source \
            name not found and no default driver specified"
    } else {
        "State: IM002, Native error: 0, Message: [unixODBC][Driver Manager]Data source name not \
            found, and no default driver specified"
    };

    let environment = Environment::new().unwrap();
    let environment = environment.set_odbc_version_3().unwrap();
    let conn = DataSource::with_parent(&environment).unwrap();
    let result = conn.connect_with_connection_string("bla");
    let message = format!("{}", result.err().unwrap());
    assert_eq!(expected, message);
}

#[test]
fn test_connection_string() {

    let environment = Environment::new().unwrap();
    let environment = environment.set_odbc_version_3().unwrap();
    let conn = DataSource::with_parent(&environment).unwrap();
    let conn = conn.connect_with_connection_string("dsn=PostgreSQL;Uid=postgres;Pwd=;")
        .unwrap();
    conn.disconnect().unwrap();
}

#[test]
fn list_drivers() {
    let environment = Environment::new().unwrap();
    let environment = environment.set_odbc_version_3().unwrap();
    let drivers = environment.drivers()
        .expect("Drivers can be iterated over");
    println!("{:?}", drivers);

    let expected = ["PostgreSQL ANSI", "PostgreSQL Unicode", "SQLite", "SQLite3"];
    assert!(drivers.iter().map(|d| &d.description).eq(expected.iter()));
}

#[test]
fn list_data_sources() {
    let environment = Environment::new().unwrap();
    let environment = environment.set_odbc_version_3().unwrap();
    let sources = environment.data_sources()
        .expect("Data sources can be iterated over");
    println!("{:?}", sources);

    let expected = [DataSourceInfo {
                        server_name: "PostgreSQL".to_owned(),
                        description: "PostgreSQL Unicode".to_owned(),
                    }];
    assert!(sources.iter().eq(expected.iter()));
}

#[test]
fn list_user_data_sources() {
    let environment = Environment::new().unwrap();
    let environment = environment.set_odbc_version_3().unwrap();
    let sources = environment.user_data_sources()
        .expect("Data sources can be iterated over");
    println!("{:?}", sources);

    let expected = [DataSourceInfo {
                        server_name: "PostgreSQL".to_owned(),
                        description: "PostgreSQL Unicode".to_owned(),
                    }];
    assert!(sources.iter().eq(expected.iter()));
}

#[test]
fn list_system_data_sources() {
    let environment = Environment::new().unwrap();
    let environment = environment.set_odbc_version_3().unwrap();
    let sources = environment.system_data_sources()
        .expect("Data sources can be iterated over");
    println!("{:?}", sources);

    let expected: [DataSourceInfo; 0] = [];
    assert!(sources.iter().eq(expected.iter()));
}

