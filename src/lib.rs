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

        let mut env = Environment::new().unwrap();
        let mut ds = DataSource::with_dsn_and_credentials(&mut env, "PostgreSQL", "postgres", "")
            .unwrap();
        let statement = Statement::with_tables(&mut ds).unwrap();
        assert_eq!(statement.num_result_cols().unwrap(), 5);
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

    #[test]
    fn it_works() {

        use raw::*;
        use raw::SQLRETURN::*;
        use std::ffi::{CStr, CString};

        let is_success = |ret| {
            match ret {
                SQL_SUCCESS |
                SQL_SUCCESS_WITH_INFO => true,
                _ => false,
            }
        };

        unsafe {
            let mut env: SQLHENV = std::ptr::null_mut();
            SQLAllocEnv(&mut env);

            let mut ret: SQLRETURN;

            let mut name = [0; 1024];
            let mut name_ret: SQLSMALLINT = 0;

            let mut desc = [0; 1024];
            let mut desc_ret: SQLSMALLINT = 0;

            println!("Driver list:");
            while is_success(SQLDrivers(env,
                                        SQL_FETCH_NEXT,
                                        name.as_mut_ptr(),
                                        name.len() as i16,
                                        &mut name_ret,
                                        desc.as_mut_ptr(),
                                        desc.len() as i16,
                                        &mut desc_ret)) {
                println!("{:?} - {:?}",
                         CStr::from_ptr(name.as_ptr() as *const i8),
                         CStr::from_ptr(desc.as_ptr() as *const i8));
            }

            println!("DataSource list:");
            while is_success(SQLDataSources(env,
                                            SQL_FETCH_NEXT,
                                            name.as_mut_ptr(),
                                            name.len() as i16,
                                            &mut name_ret,
                                            desc.as_mut_ptr(),
                                            desc.len() as i16,
                                            &mut desc_ret)) {
                println!("{:?} - {:?}",
                         CStr::from_ptr(name.as_ptr() as *const i8),
                         CStr::from_ptr(desc.as_ptr() as *const i8));
            }

            let mut dbc: SQLHDBC = std::ptr::null_mut();
            SQLAllocConnect(env, &mut dbc);

            let dsn = CString::new("DSN=pglocal;Database=crm;Uid=postgres;Password=postgres")
                .unwrap();

            println!("CONNECTING TO {:?}", dsn);

            let dsn_ptr = dsn.into_raw();

            ret = SQLDriverConnect(dbc,
                                   std::ptr::null_mut(),
                                   dsn_ptr as *mut u8,
                                   SQL_NTS,
                                   name.as_mut_ptr(),
                                   name.len() as i16,
                                   &mut name_ret,
                                   SQL_DRIVER_NOPROMPT);

            CString::from_raw(dsn_ptr);

            if is_success(ret) {
                println!("CONNECTED: {:?}",
                         CStr::from_ptr(name.as_ptr() as *const i8));

                let mut stmt: SQLHSTMT = std::ptr::null_mut();
                SQLAllocStmt(dbc, &mut stmt);

                let sql = CString::new("select * from security.user").unwrap();

                println!("EXECUTING SQL {:?}", sql);

                let sql_ptr = sql.into_raw();
                ret = SQLExecDirect(stmt, sql_ptr as *mut u8, SQL_NTSL);
                CString::from_raw(sql_ptr);

                if is_success(ret) {
                    let mut columns: SQLSMALLINT = 0;
                    SQLNumResultCols(stmt, &mut columns);

                    println!("SUCCESSFUL:");

                    let mut i = 1;
                    while is_success(SQLFetch(stmt)) {
                        println!("\tROW: {}", i);

                        for j in 1..columns {
                            let mut indicator: SQLLEN = 0;
                            let mut buf = [0; 512];
                            ret = SQLGetData(stmt,
                                             j as u16,
                                             1,
                                             buf.as_mut_ptr() as SQLPOINTER,
                                             buf.len() as SQLLEN,
                                             &mut indicator);
                            if is_success(ret) {
                                if indicator == -1 {
                                    println!("Column {}: NULL", j);
                                } else {
                                    println!("Column {}: {:?}",
                                             j,
                                             CStr::from_ptr(buf.as_ptr() as *const i8));
                                }
                            }
                        }

                        i += 1;
                    }
                } else {
                    println!("FAILED:");
                    let mut i = 1;
                    let mut native: SQLINTEGER = 0;
                    while is_success(SQLGetDiagRec(SQL_HANDLE_STMT,
                                                   stmt,
                                                   i,
                                                   name.as_mut_ptr(),
                                                   &mut native,
                                                   desc.as_mut_ptr(),
                                                   desc.len() as i16,
                                                   &mut desc_ret)) {
                        println!("\t{:?}:{}:{}:{:?}",
                                 CStr::from_ptr(name.as_ptr() as *const i8),
                                 i,
                                 native,
                                 CStr::from_ptr(desc.as_ptr() as *const i8));
                        i += 1;
                    }
                }

                SQLFreeHandle(SQL_HANDLE_STMT, stmt);
                SQLDisconnect(dbc);
            } else {
                println!("NOT CONNECTED: {:?}",
                         CStr::from_ptr(name.as_ptr() as *const i8));
                let mut i = 1;
                let mut native: SQLINTEGER = 0;
                while is_success(SQLGetDiagRec(SQL_HANDLE_DBC,
                                               dbc,
                                               i,
                                               name.as_mut_ptr(),
                                               &mut native,
                                               desc.as_mut_ptr(),
                                               desc.len() as i16,
                                               &mut desc_ret)) {
                    println!("\t{:?}:{}:{}:{:?}",
                             CStr::from_ptr(name.as_ptr() as *const i8),
                             i,
                             native,
                             CStr::from_ptr(desc.as_ptr() as *const i8));
                    i += 1;
                }
            }

            SQLFreeConnect(dbc);
            SQLFreeEnv(env);
        }

        println!("BYE!");
    }
}