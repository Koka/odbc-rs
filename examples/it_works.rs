extern crate odbc;
use odbc::*;

fn main() {

    match connect() {
        Ok(()) => (),
        Err(err) => println!("{}", err),
    }
}

fn connect() -> odbc::Result<()> {

    let mut env = Environment::new().unwrap();
    env.set_odbc_version_3()?;
    println!("Driver list:");
    for driver_info in env.drivers()? {
        println!("\nDriver Name: {}", driver_info.description);
        for (key, value) in driver_info.attributes {
            println!("    {}={}", key, value);
        }
    }

    println!("\nDataSource list:");
    for ds in env.data_sources()? {
        println!("    name: {} description: {}",
                 ds.server_name,
                 ds.description);
    }

    use ffi::*;
    use std::ffi::{CStr, CString};

    let is_success = |ret| match ret {
        SQL_SUCCESS |
        SQL_SUCCESS_WITH_INFO => true,
        _ => false,
    };

    unsafe {

        let mut ret: SQLRETURN;

        let mut name = [0; 1024];
        let mut name_ret: SQLSMALLINT = 0;

        let mut desc = [0; 1024];
        let mut desc_ret: SQLSMALLINT = 0;

        let mut dbc: SQLHANDLE = std::ptr::null_mut();
        SQLAllocHandle(SQL_HANDLE_DBC,
                       env.handle() as SQLHANDLE,
                       &mut dbc as *mut SQLHANDLE);
        let mut dbc = dbc as SQLHDBC;

        let dsn = CString::new("DSN=pglocal;Database=crm;Uid=postgres;Password=postgres").unwrap();

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

            let mut stmt: SQLHANDLE = std::ptr::null_mut();
            SQLAllocHandle(SQL_HANDLE_STMT,
                           dbc as SQLHANDLE,
                           &mut stmt as *mut SQLHANDLE);
            let mut stmt = stmt as SQLHSTMT;

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
                                         SQL_C_CHAR,
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
                                               stmt as SQLHANDLE,
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

            SQLFreeHandle(SQL_HANDLE_STMT, stmt as SQLHANDLE);
            SQLDisconnect(dbc);
        } else {
            println!("NOT CONNECTED: {:?}",
                     CStr::from_ptr(name.as_ptr() as *const i8));
            let mut i = 1;
            let mut native: SQLINTEGER = 0;
            while is_success(SQLGetDiagRec(SQL_HANDLE_DBC,
                                           dbc as SQLHANDLE,
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

        SQLFreeHandle(SQL_HANDLE_DBC, dbc as SQLHANDLE);
    }

    println!("BYE!");
    Ok(())
}

