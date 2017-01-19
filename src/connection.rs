//! Holds implementation of odbc connection
use super::{raw, Environment, Result, DiagRec, Error};
use std;
use std::marker::PhantomData;

/// Handle to an ODBC Connection
///
/// Connections are used to connect to data sources
pub struct Connection<'a> {
    handle: raw::SQLHDBC,
    // we use phantom data to tell the borrow checker that we need to keep the environment alive for
    // the lifetime of the connection
    env: PhantomData<&'a Environment>,
}

impl<'a> Connection<'a> {
    /// Creates a new `Connection` to a data source
    ///
    /// # Arguments
    /// * `env` - Environment used to allocate the connection handle.
    /// * `dsn` - Data source name configured in the `odbc.ini` file!()
    /// * `usr` - User identifier
    /// * `pwd` - Authentication (usually password)
    pub fn with_dsn_and_credentials<'b>(env: &'b mut Environment,
                                        dsn: &str,
                                        usr: &str,
                                        pwd: &str)
                                        -> Result<Connection<'b>> {
        let connection = Self::allocate(env)?;

        unsafe {
            match raw::SQLConnect(connection.handle,
                                  dsn.as_ptr(),
                                  dsn.as_bytes().len() as raw::SQLSMALLINT,
                                  usr.as_ptr(),
                                  usr.as_bytes().len() as raw::SQLSMALLINT,
                                  pwd.as_ptr(),
                                  pwd.as_bytes().len() as raw::SQLSMALLINT) {
                raw::SQL_SUCCESS |
                raw::SQL_SUCCESS_WITH_INFO => Ok(connection),
                _ => Err(Error::SqlError(DiagRec::create(raw::SQL_HANDLE_DBC, connection.handle))),
            }
        }
    }

    fn allocate(env: &mut Environment) -> Result<Connection> {
        unsafe {
            let mut conn = std::ptr::null_mut();
            match raw::SQLAllocHandle(raw::SQL_HANDLE_DBC, env.raw(), &mut conn) {
                raw::SQL_SUCCESS => {
                    Ok(Connection {
                        handle: conn,
                        env: PhantomData,
                    })
                }
                raw::SQL_SUCCESS_WITH_INFO => {
                    Ok(Connection {
                        handle: conn,
                        env: PhantomData,
                    })
                }
                // Driver Manager failed to allocate environment
                raw::SQL_ERROR => {
                    Err(Error::SqlError(DiagRec::create(raw::SQL_HANDLE_ENV, env.raw())))
                }
                _ => unreachable!(),
            }
        }
    }
}

impl<'a> Drop for Connection<'a> {
    fn drop(&mut self) {
        unsafe {
            raw::SQLFreeHandle(raw::SQL_HANDLE_DBC, self.handle);
        }
    }
}