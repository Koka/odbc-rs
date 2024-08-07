extern crate odbc;
extern crate odbc_safe;

use odbc::*;
use odbc_safe::AutocommitOn;

#[test]
fn list_tables() {
    let env = create_environment_v3().unwrap();
    let ds = env.connect("TestDataSource", "", "").unwrap();
    // scope is required (for now) to close statement before disconnecting
    {
        let statement = Statement::with_parent(&ds).unwrap();
        let mut statement = statement.tables_str("%", "%", "MOV%", "TABLE").unwrap();
        assert_eq!(statement.num_result_cols().unwrap(), 5);
        let rs = statement.fetch().unwrap();
        assert!(rs.is_some());
        let mut cur = rs.unwrap();
        assert_eq!(cur.get_data::<String>(1).unwrap(), None);
        assert_eq!(cur.get_data::<String>(2).unwrap(), None);
        assert_eq!(
            cur.get_data::<String>(3).unwrap(),
            Some("MOVIES".to_owned())
        );
        assert_eq!(cur.get_data::<String>(4).unwrap(), Some("TABLE".to_owned()));
        assert_eq!(cur.get_data::<String>(5).unwrap(), None);
    }
    ds.disconnect().unwrap();
}

#[test]
fn not_read_only() {
    let env = create_environment_v3().unwrap();
    let mut conn = env.connect("TestDataSource", "", "").unwrap();

    assert!(!conn.is_read_only().unwrap());
    conn.disconnect().unwrap();
}

#[test]
fn implicit_disconnect() {
    let env = create_environment_v3().unwrap();
    env.connect("TestDataSource", "", "").unwrap();

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

    let environment = create_environment_v3().unwrap();
    let result = environment.connect_with_connection_string("bla");
    let message = format!("{}", result.err().unwrap());
    assert_eq!(expected, message);
}

#[test]
fn test_connection_string() {
    let environment = create_environment_v3().unwrap();
    let conn = environment
        .connect_with_connection_string("dsn=TestDataSource;Uid=;Pwd=;")
        .unwrap();
    conn.disconnect().unwrap();
}

#[test]
fn test_direct_select() {
    let env = create_environment_v3().unwrap();
    let conn = env.connect("TestDataSource", "", "").unwrap();
    let stmt = Statement::with_parent(&conn).unwrap();

    let mut stmt = match stmt
        .exec_direct("SELECT TITLE, YEAR FROM MOVIES ORDER BY YEAR")
        .unwrap()
    {
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

    let check = actual
        == vec![
            Movie {
                title: "2001: A Space Odyssey".to_owned(),
                year: 1968,
            },
            Movie {
                title: "Jurassic Park".to_owned(),
                year: 1993,
            },
        ];

    println!("test_direct_select query result: {:?}", actual);

    assert!(check);
}

#[test]
fn reuse_statement() {
    let env = create_environment_v3().unwrap();
    let conn = env.connect("TestDataSource", "", "").unwrap();
    let stmt = Statement::with_parent(&conn).unwrap();

    let stmt = match stmt
        .exec_direct("CREATE TABLE STAGE (A VARCHAR, B VARCHAR);")
        .unwrap()
    {
        Data(stmt) => stmt.close_cursor().unwrap(), //A result set has been returned, we need to close it.
        NoData(stmt) => stmt,
    };
    let stmt = match stmt
        .exec_direct("INSERT INTO STAGE (A, B) VALUES ('Hello', 'World');")
        .unwrap()
    {
        Data(stmt) => stmt.close_cursor().unwrap(),
        NoData(stmt) => stmt,
    };
    if let Data(mut stmt) = stmt.exec_direct("SELECT A, B FROM STAGE;").unwrap() {
        {
            let mut cursor = stmt.fetch().unwrap().unwrap();
            assert_eq!(cursor.get_data::<String>(1).unwrap().unwrap(), "Hello");
            assert_eq!(cursor.get_data::<String>(2).unwrap().unwrap(), "World");
        }
        let stmt = stmt.close_cursor().unwrap();
        stmt.exec_direct("DROP TABLE STAGE;").unwrap();
    } else {
        panic!("SELECT statement returned no result set")
    };
}

#[test]
fn execution_with_parameter() {
    let env = create_environment_v3().unwrap();
    let conn = env.connect("TestDataSource", "", "").unwrap();
    let stmt = Statement::with_parent(&conn).unwrap();
    let param = 1968;
    let stmt = stmt.bind_parameter(1, &param).unwrap();

    if let Data(mut stmt) = stmt
        .exec_direct("SELECT TITLE FROM MOVIES WHERE YEAR = ?")
        .unwrap()
    {
        let mut cursor = stmt.fetch().unwrap().unwrap();
        assert_eq!(
            cursor.get_data::<String>(1).unwrap().unwrap(),
            "2001: A Space Odyssey"
        );
    } else {
        panic!("SELECT statement returned no result set")
    };
}

#[test]
fn prepared_execution() {
    let env = create_environment_v3().unwrap();
    let conn = env.connect("TestDataSource", "", "").unwrap();
    let stmt = Statement::with_parent(&conn).unwrap();
    let stmt = stmt
        .prepare("SELECT TITLE FROM MOVIES WHERE YEAR = ?")
        .unwrap();

    fn execute_query<'a>(
        year: u16,
        expected: &str,
        stmt: Statement<'a, 'a, Prepared, NoResult, AutocommitOn>,
    ) -> Result<Statement<'a, 'a, Prepared, NoResult, AutocommitOn>> {
        let stmt = stmt.bind_parameter(1, &year)?;
        let stmt = if let Data(mut stmt) = stmt.execute()? {
            {
                let mut cursor = stmt.fetch()?.unwrap();
                assert_eq!(cursor.get_data::<String>(1)?.unwrap(), expected);
            }
            stmt.close_cursor()?
        } else {
            panic!("SELECT statement returned no result set");
        };
        stmt.reset_parameters()
    }

    let stmt = execute_query(1968, "2001: A Space Odyssey", stmt).unwrap();
    execute_query(1993, "Jurassic Park", stmt).unwrap();
}

// These tests query the results of catalog functions. These results are only likely to match the
// expectation on the travis ci build on linux. Therefore we limit compilation and execution of
// these tests to this platform.
#[cfg_attr(not(feature = "travis"), ignore)]
#[test]
fn list_drivers() {
    let mut environment = create_environment_v3().unwrap();
    let drivers = environment.drivers().expect("Drivers can be iterated over");
    println!("{:?}", drivers);

    let expected = ["PostgreSQL ANSI", "PostgreSQL Unicode", "SQLite", "SQLite3"];
    let mut actual: Vec<_> = drivers.iter().map(|d| &d.description).collect();
    actual.sort();
    assert_eq!(actual, expected);
}

#[cfg_attr(not(feature = "travis"), ignore)]
#[test]
fn list_data_sources() {
    let mut environment = create_environment_v3().unwrap();
    let sources = environment
        .data_sources()
        .expect("Data sources can be iterated over");
    println!("{:?}", sources);

    let expected = [
        DataSourceInfo {
            server_name: "PostgreSQL".to_owned(),
            driver: "PostgreSQL Unicode".to_owned(),
        },
        DataSourceInfo {
            server_name: "TestDataSource".to_owned(),
            driver: "SQLite3".to_owned(),
        },
    ];
    assert!(sources.iter().eq(expected.iter()));
}

#[cfg_attr(not(feature = "travis"), ignore)]
#[test]
fn list_user_data_sources() {
    let mut environment = create_environment_v3().unwrap();
    let sources = environment
        .user_data_sources()
        .expect("Data sources can be iterated over");
    println!("{:?}", sources);

    let expected = [
        DataSourceInfo {
            server_name: "PostgreSQL".to_owned(),
            driver: "PostgreSQL Unicode".to_owned(),
        },
        DataSourceInfo {
            server_name: "TestDataSource".to_owned(),
            driver: "SQLite3".to_owned(),
        },
    ];
    assert!(sources.iter().eq(expected.iter()));
}

#[cfg_attr(not(feature = "travis"), ignore)]
#[test]
fn list_system_data_sources() {
    let mut environment = create_environment_v3().unwrap();
    let sources = environment
        .system_data_sources()
        .expect("Data sources can be iterated over");
    println!("{:?}", sources);

    let expected: [DataSourceInfo; 0] = [];
    assert!(sources.iter().eq(expected.iter()));
}

#[test]
fn read_big_string() {
    let env = create_environment_v3().unwrap();
    let conn = env.connect("TestDataSource", "", "").unwrap();
    let stmt = Statement::with_parent(&conn).unwrap();

    let stmt = match stmt
        .exec_direct("CREATE TABLE READ_BIG_STRING (DATA TEXT)")
        .unwrap()
    {
        Data(stmt) => stmt.close_cursor().unwrap(),
        NoData(stmt) => stmt,
    };
    let data = "Hello, World".repeat(43);
    assert!(data.len() > 512);
    stmt.prepare("INSERT INTO READ_BIG_STRING VALUES (?)")
        .unwrap()
        .bind_parameter(1, &data)
        .unwrap()
        .execute()
        .unwrap();
    let stmt = Statement::with_parent(&conn).unwrap();
    let sel_query = "SELECT DATA FROM READ_BIG_STRING";
    let stmt = if let Data(mut stmt) = stmt.exec_direct(sel_query).unwrap() {
        {
            let mut cursor = stmt.fetch().unwrap().unwrap();
            // Do read with bytes buffer
            let data0 = cursor.get_data::<Vec<u8>>(1).unwrap().unwrap();
            assert_eq!(data0, data.as_bytes());
        }
        stmt.close_cursor().unwrap()
    } else {
        panic!("SELECT statement returned no result set")
    };
    let stmt = if let Data(mut stmt) = stmt.exec_direct(sel_query).unwrap() {
        {
            let mut cursor = stmt.fetch().unwrap().unwrap();
            // Do read with String buffer
            let data0 = cursor.get_data::<String>(1).unwrap().unwrap();
            assert_eq!(data0, data);
        }
        stmt.close_cursor().unwrap()
    } else {
        panic!("SELECT statement returned no result set")
    };
    stmt.exec_direct("DROP TABLE READ_BIG_STRING").unwrap();
}

#[test]
fn zero_truncation_bug() {
    let env = create_environment_v3().unwrap();
    let conn = env.connect("TestDataSource", "", "").unwrap();
    let stmt = Statement::with_parent(&conn).unwrap();

    let stmt = match stmt
        .exec_direct("CREATE TABLE ZERO_TRUNCATION_BUG (DATA BLOB);")
        .unwrap()
    {
        Data(stmt) => stmt.close_cursor().unwrap(),
        NoData(stmt) => stmt,
    };
    // Reproduction of zeroes truncation bug. Until now there is no chance to query binary data
    // with zero at 512 byte border.
    let data = vec![0; 513];
    stmt.prepare("INSERT INTO ZERO_TRUNCATION_BUG VALUES (?)")
        .unwrap()
        .bind_parameter(1, &data)
        .unwrap()
        .execute()
        .unwrap();
    let stmt = Statement::with_parent(&conn).unwrap();
    if let Data(mut stmt) = stmt
        .exec_direct("SELECT DATA FROM ZERO_TRUNCATION_BUG")
        .unwrap()
    {
        {
            let mut cursor = stmt.fetch().unwrap().unwrap();
            let data0 = cursor.get_data::<Vec<u8>>(1).unwrap().unwrap();
            assert_eq!(data0, data);
        }
        let stmt = stmt.close_cursor().unwrap();
        stmt.exec_direct("DROP TABLE ZERO_TRUNCATION_BUG").unwrap();
    } else {
        panic!("SELECT statement returned no result set")
    };
}
