//! Holds implementation of odbc connection
use super::{raw, Environment, Result, Error};
use safe::{Handle, GetDiagRec};
use super::raw::SQLRETURN::*;
use std;
use std::marker::PhantomData;

/// Represents a connection to an ODBC data source
pub struct DataSource<'a> {
    handle: raw::SQLHDBC,
    // we use phantom data to tell the borrow checker that we need to keep the environment alive for
    // the lifetime of the connection
    env: PhantomData<&'a Environment>,
}

impl<'a> DataSource<'a> {
    /// Connects to an ODBC data source
    ///
    /// # Arguments
    /// * `env` - Environment used to allocate the data source handle.
    /// * `dsn` - Data source name configured in the `odbc.ini` file!()
    /// * `usr` - User identifier
    /// * `pwd` - Authentication (usually password)
    pub fn with_dsn_and_credentials<'b>(env: &'b mut Environment,
                                        dsn: &str,
                                        usr: &str,
                                        pwd: &str)
                                        -> Result<DataSource<'b>> {
        let data_source = Self::allocate(env)?;

        unsafe {
            match raw::SQLConnect(data_source.handle,
                                  dsn.as_ptr(),
                                  dsn.as_bytes().len() as raw::SQLSMALLINT,
                                  usr.as_ptr(),
                                  usr.as_bytes().len() as raw::SQLSMALLINT,
                                  pwd.as_ptr(),
                                  pwd.as_bytes().len() as raw::SQLSMALLINT) {
                SQL_SUCCESS |
                SQL_SUCCESS_WITH_INFO => Ok(data_source),
                _ => Err(Error::SqlError(data_source.get_diagnostic_record(1).unwrap())),
            }
        }
    }

    /// `true` if the data source is set to READ ONLY mode, `false` otherwise.
    ///
    /// This characteristic pertains only to the data source itself; it is not characteristic of
    /// the driver that enables access to the data source. A driver that is read/write can be used
    /// with a data source that is read-only. If a driver is read-only, all of its data sources
    /// must be read-only.
    pub fn read_only(&self) -> Result<bool> {
        let mut buffer: [u8; 2] = [0; 2];

        unsafe {
            match raw::SQLGetInfo(self.handle,
                                  raw::SQL_DATA_SOURCE_READ_ONLY,
                                  buffer.as_mut_ptr() as *mut std::os::raw::c_void,
                                  buffer.len() as raw::SQLSMALLINT,
                                  std::ptr::null_mut()) {
                SQL_SUCCESS |
                SQL_SUCCESS_WITH_INFO => {
                    Ok({
                        assert!(buffer[1] == 0);
                        match buffer[0] as char {
                            'N' => false,
                            'Y' => true,
                            _ => panic!(r#"Driver may only return "N" or "Y""#),
                        }
                    })
                }
                SQL_ERROR => Err(Error::SqlError(self.get_diagnostic_record(1).unwrap())),
                _ => unreachable!(),
            }
        }
    }

    /// Allows access to the raw ODBC handle
    pub unsafe fn raw(&mut self) -> raw::SQLHDBC {
        self.handle
    }

    fn allocate(env: &mut Environment) -> Result<DataSource> {
        unsafe {
            let mut conn = std::ptr::null_mut();
            match raw::SQLAllocHandle(raw::SQL_HANDLE_DBC, env.raw(), &mut conn) {
                SQL_SUCCESS |
                SQL_SUCCESS_WITH_INFO => {
                    Ok(DataSource {
                        handle: conn,
                        env: PhantomData,
                    })
                }
                // Driver Manager failed to allocate environment
                SQL_ERROR => Err(Error::SqlError(env.get_diagnostic_record(1).unwrap())),
                _ => unreachable!(),
            }
        }
    }
}

unsafe impl<'a> Handle for DataSource<'a> {
    fn handle(&self) -> raw::SQLHANDLE {
        self.handle
    }

    fn handle_type() -> raw::SQLSMALLINT {
        raw::SQL_HANDLE_DBC
    }
}

impl<'a> Drop for DataSource<'a> {
    fn drop(&mut self) {
        unsafe {
            raw::SQLFreeHandle(raw::SQL_HANDLE_DBC, self.handle);
        }
    }
}
