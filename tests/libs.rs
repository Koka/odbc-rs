extern crate odbc;
use odbc::*;
mod native_types;

#[test]
fn list_tables() {

    let env = Environment::new().unwrap();
    let env = env.set_odbc_version_3().unwrap();
    let ds = DataSource::with_parent(&env).unwrap();
    let mut ds = ds.connect("TestDataSource", "", "").unwrap();
    // scope is required (for now) to close statement before disconnecting
    {
        let statement = Statement::with_parent(&mut ds).unwrap();
        let statement = statement.tables().unwrap();
        assert_eq!(statement.num_result_cols().unwrap(), 5);
    }
    ds.disconnect().unwrap();
}

#[test]
fn not_read_only() {

    let env = Environment::new().unwrap();
    let env = env.set_odbc_version_3().unwrap();
    let conn = DataSource::with_parent(&env).unwrap();
    let conn = conn.connect("TestDataSource", "", "").unwrap();

    assert!(!conn.read_only().unwrap());
    conn.disconnect().unwrap();
}

#[test]
fn implicit_disconnect() {

    let env = Environment::new().unwrap();
    let env = env.set_odbc_version_3().unwrap();
    let conn = DataSource::with_parent(&env).unwrap();
    conn.connect("TestDataSource", "", "").unwrap();

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
    let conn = conn.connect_with_connection_string("dsn=TestDataSource;Uid=;Pwd=;")
        .unwrap();
    conn.disconnect().unwrap();
}

#[test]
fn test_direct_select() {
    let env = Environment::new().unwrap().set_odbc_version_3().unwrap();
    let conn = DataSource::with_parent(&env).unwrap().connect("TestDataSource", "", "").unwrap();
    let stmt = Statement::with_parent(&conn).unwrap();

    let mut stmt = match stmt.exec_direct("SELECT TITLE, YEAR FROM MOVIES ORDER BY YEAR")
        .unwrap() {
        Data(stmt) => stmt,
        NoData(_) => panic!("SELECT statement did not return result set!"),
    };

    assert_eq!(stmt.num_result_cols().unwrap(), 2);

    #[derive(PartialEq, Debug)]
    struct Movie {
        title: String,
        year: i16,
    }

    let mut actual = Vec::new();
    while let Some(mut cursor) = stmt.fetch().unwrap() {
        actual.push(Movie {
            title: cursor.get_data(1).unwrap().unwrap(),
            year: cursor.get_data(2).unwrap().unwrap(),
        })
    }

    let check = actual ==
                vec![Movie {
                         title: "2001: A Space Odyssey".to_owned(),
                         year: 1968,
                     },
                     Movie {
                         title: "Jurassic Park".to_owned(),
                         year: 1993,
                     }];

    println!("test_direct_select query result: {:?}", actual);

    assert!(check);
}

#[test]
fn reuse_statement() {
    let env = Environment::new().unwrap().set_odbc_version_3().unwrap();
    let conn = DataSource::with_parent(&env).unwrap().connect("TestDataSource", "", "").unwrap();
    let stmt = Statement::with_parent(&conn).unwrap();

    let stmt = match stmt.exec_direct("CREATE TABLE STAGE (A VARCHAR, B VARCHAR);").unwrap() {
        Data(stmt) => stmt.close_cursor().unwrap(), //A result set has been returned, we need to close it.
        NoData(stmt) => stmt,
    };
    let stmt = match stmt.exec_direct("INSERT INTO STAGE (A, B) VALUES ('Hello', 'World');")
        .unwrap() {
        Data(stmt) => stmt.close_cursor().unwrap(),
        NoData(stmt) => stmt,
    };
    if let Data(mut stmt) = stmt.exec_direct("SELECT A, B FROM STAGE;").unwrap() {
        {
            let mut cursor = stmt.fetch().unwrap().unwrap();
            assert!(cursor.get_data::<String>(1).unwrap().unwrap() == "Hello");
            assert!(cursor.get_data::<String>(2).unwrap().unwrap() == "World");
        }
        let stmt = stmt.close_cursor().unwrap();
        stmt.exec_direct("DROP TABLE STAGE;").unwrap();
    } else {
        panic!("SELECT statement returned no result set")
    };
}

#[test]
fn execution_with_parameter() {
    let env = Environment::new().unwrap().set_odbc_version_3().unwrap();
    let conn = DataSource::with_parent(&env).unwrap().connect("TestDataSource", "", "").unwrap();
    let stmt = Statement::with_parent(&conn).unwrap();
    let param = 1968;
    let stmt = stmt.bind_parameter(1, &param).unwrap();

    if let Data(mut stmt) = stmt.exec_direct("SELECT TITLE FROM MOVIES WHERE YEAR = ?").unwrap() {
        let mut cursor = stmt.fetch().unwrap().unwrap();
        assert!(cursor.get_data::<String>(1).unwrap().unwrap() == "2001: A Space Odyssey");
    } else {
        panic!("SELECT statement returned no result set")
    };
}

// These tests query the results of catalog functions. These results are only likely to match the
// expectation on the travis.ci build on linux. Therefore we limit compilation and execution of
// these tests to this platform.
#[cfg_attr(not(feature = "travis"), ignore)]
#[test]
fn list_drivers() {
    let environment = Environment::new().unwrap();
    let environment = environment.set_odbc_version_3().unwrap();
    let drivers = environment.drivers()
        .expect("Drivers can be iterated over");
    println!("{:?}", drivers);

    let expected = ["PostgreSQL ANSI", "PostgreSQL Unicode", "SQLite", "SQLite3"];
    assert!(drivers.iter().map(|d| &d.description).contains(expected.iter()));
}

#[cfg_attr(not(feature = "travis"), ignore)]
#[test]
fn list_data_sources() {
    let environment = Environment::new().unwrap();
    let environment = environment.set_odbc_version_3().unwrap();
    let sources = environment.data_sources()
        .expect("Data sources can be iterated over");
    println!("{:?}", sources);

    let expected = [DataSourceInfo {
                        server_name: "PostgreSQL".to_owned(),
                        driver: "PostgreSQL Unicode".to_owned(),
                    },
                    DataSourceInfo {
                        server_name: "TestDataSource".to_owned(),
                        driver: "SQLite3".to_owned(),
                    }];
    assert!(sources.iter().contains(expected.iter()));
}

#[cfg_attr(not(feature = "travis"), ignore)]
#[test]
fn list_user_data_sources() {
    let environment = Environment::new().unwrap();
    let environment = environment.set_odbc_version_3().unwrap();
    let sources = environment.user_data_sources()
        .expect("Data sources can be iterated over");
    println!("{:?}", sources);

    let expected = [DataSourceInfo {
                        server_name: "PostgreSQL".to_owned(),
                        driver: "PostgreSQL Unicode".to_owned(),
                    },
                    DataSourceInfo {
                        server_name: "TestDataSource".to_owned(),
                        driver: "SQLite3".to_owned(),
                    }];
    assert!(sources.iter().contains(expected.iter()));
}

#[cfg_attr(not(feature = "travis"), ignore)]
#[test]
fn list_system_data_sources() {
    let environment = Environment::new().unwrap();
    let environment = environment.set_odbc_version_3().unwrap();
    let sources = environment.system_data_sources()
        .expect("Data sources can be iterated over");
    println!("{:?}", sources);

    let expected: [DataSourceInfo; 0] = [];
    assert!(sources.iter().contains(expected.iter()));
}
