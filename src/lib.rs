pub mod raw;

mod error;
pub use error::*;
mod environment;
pub use environment::*;
mod data_source;
pub use data_source::*;
mod statement;
pub use statement::*;

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn list_tables() {
        use super::raw;

        let mut env = Environment::new().unwrap();
        let mut ds = DataSource::with_dsn_and_credentials(&mut env, "PostgreSQL", "postgres", "")
            .unwrap();
        let statement = Statement::with_tables(&mut ds).unwrap();
        assert_eq!(statement.num_result_cols().unwrap(), 4);
    }

    #[test]
    fn test_connection() {

        let mut environment = Environment::new().expect("Environment can be created");
        let conn =
            DataSource::with_dsn_and_credentials(&mut environment, "PostgreSQL", "postgres", "")
                .expect("Could not connect");

        assert!(!conn.read_only().unwrap());
    }

    #[test]
    fn list_drivers() {
        let environment = Environment::new();
        let drivers = environment.expect("Environment can be created")
            .drivers()
            .expect("Drivers can be iterated over");
        println!("{:?}", drivers);

        let expected = ["PostgreSQL ANSI", "PostgreSQL Unicode", "SQLite", "SQLite3"];
        assert!(drivers.iter().map(|d| &d.description).eq(expected.iter()));
    }

    #[test]
    fn list_data_sources() {
        let environment = Environment::new();
        let sources = environment.expect("Environment can be created")
            .data_sources()
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
        let environment = Environment::new();
        let sources = environment.expect("Environment can be created")
            .user_data_sources()
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
        let environment = Environment::new();
        let sources = environment.expect("Environment can be created")
            .system_data_sources()
            .expect("Data sources can be iterated over");
        println!("{:?}", sources);

        let expected: [DataSourceInfo; 0] = [];
        assert!(sources.iter().eq(expected.iter()));
    }

    #[test]
    fn provoke_error() {
        use std;
        let mut environment = Environment::new().unwrap();
        // let mut dbc: raw::SQLHDBC = 0;
        let error;
        unsafe {
            // We set the output pointer to zero. This is an error!
            raw::SQLAllocHandle(raw::SQL_HANDLE_DBC, environment.raw(), std::ptr::null_mut());
            // Let's create a diagnostic record describing that error
            error = Error::SqlError(DiagRec::create(raw::SQL_HANDLE_ENV, environment.raw()));
        }
        if cfg!(target_os = "windows") {
            assert_eq!(format!("{}", error),
                       "[Microsoft][ODBC Driver Manager] Invalid argument value");
        } else {
            assert_eq!(format!("{}", error),
                       "[unixODBC][Driver Manager]Invalid use of null pointer");
        }
    }
}