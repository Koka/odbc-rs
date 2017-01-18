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
    pub fn with_dsn(env: &mut Environment) -> Result<Connection> {
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