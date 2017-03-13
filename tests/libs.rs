extern crate odbc;
use odbc::*;

#[test]
fn list_tables() {

    let mut env = Environment::new().unwrap();
    env.set_odbc_version_3().unwrap();
    let mut ds = DataSource::with_parent(&mut env).unwrap();
    ds.connect("PostgreSQL", "postgres", "").unwrap();
    // scope is required (for now) to close statement before disconnecting
    {
        let mut statement = Statement::with_parent(&mut ds).unwrap();
        statement.tables().unwrap();
        assert_eq!(statement.num_result_cols().unwrap(), 5);
    }
    ds.disconnect().unwrap();
}

#[test]
fn test_connection() {

    let mut environment = Environment::new().expect("Environment can be created");
    environment.set_odbc_version_3().unwrap();
    let mut conn = DataSource::with_parent(&mut environment).unwrap();
    conn.connect("PostgreSQL", "postgres", "").unwrap();

    assert!(!conn.read_only().unwrap());
    conn.disconnect().unwrap();
}

#[test]
fn test_invalid_connection_string() {

    let expected = if cfg!(target_os = "windows") {
        "State: IM002, Native error: 0, Message: [Microsoft][ODBC Driver Manager] Data source \
            name not found and no default driver specified"
    } else {
        "State: IM002, Native error: 0, Message: [unixODBC][Driver Manager]Data source name not \
            found, and no default driver specified"
    };

    let mut environment = Environment::new().unwrap();
    environment.set_odbc_version_3().unwrap();
    let mut conn = DataSource::with_parent(&environment).unwrap();
    let result = conn.connect_with_connection_string("bla");
    let message = format!("{}", result.err().unwrap());
    assert_eq!(expected, message);
}

#[test]
fn test_connection_string() {

    let mut environment = Environment::new().unwrap();
    environment.set_odbc_version_3().unwrap();
    let mut conn = DataSource::with_parent(&environment).unwrap();
    conn.connect_with_connection_string("dsn=PostgreSQL;Uid=postgres;Pwd=;")
        .unwrap();
    conn.disconnect().unwrap();
}

#[test]
fn list_drivers() {
    let mut environment = Environment::new().unwrap();
    environment.set_odbc_version_3().unwrap();
    let drivers = environment.drivers()
        .expect("Drivers can be iterated over");
    println!("{:?}", drivers);

    let expected = ["PostgreSQL ANSI", "PostgreSQL Unicode", "SQLite", "SQLite3"];
    assert!(drivers.iter().map(|d| &d.description).eq(expected.iter()));
}

#[test]
fn list_data_sources() {
    let mut environment = Environment::new().unwrap();
    environment.set_odbc_version_3().unwrap();
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
    let mut environment = Environment::new().unwrap();
    environment.set_odbc_version_3().unwrap();
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
    let mut environment = Environment::new().unwrap();
    environment.set_odbc_version_3().unwrap();
    let sources = environment.system_data_sources()
        .expect("Data sources can be iterated over");
    println!("{:?}", sources);

    let expected: [DataSourceInfo; 0] = [];
    assert!(sources.iter().eq(expected.iter()));
}

