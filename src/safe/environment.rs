use super::{as_buffer_length, Handle};
use raw::{SQLAllocHandle, SQLFreeHandle, SQLSetEnvAttr, SQLDataSources, SQLDrivers, SQLRETURN,
          SQLHENV, SQLHANDLE, SQLSMALLINT, SQLUSMALLINT, SQLCHAR, SQL_HANDLE_ENV,
          SQL_ATTR_ODBC_VERSION, SQL_OV_ODBC3};
use std::ptr::null_mut;
use std::os::raw::c_void;

/// Safe wrapper around ODBC Environment handle
pub struct Environment {
    handle: SQLHENV,
}

impl Drop for Environment {
    fn drop(&mut self) {
        unsafe {
            SQLFreeHandle(SQL_HANDLE_ENV, self.handle);
        }
    }
}

unsafe impl Handle for Environment {
    fn handle(&self) -> SQLHANDLE {
        self.handle
    }

    fn handle_type() -> SQLSMALLINT {
        SQL_HANDLE_ENV
    }
}

impl Environment {
    pub fn new() -> EnvAllocResult {

        use self::EnvAllocResult::*;

        let mut env = null_mut();
        match unsafe { SQLAllocHandle(SQL_HANDLE_ENV, null_mut(), &mut env) } {
            SQLRETURN::SQL_SUCCESS => Success(Environment { handle: env }),
            SQLRETURN::SQL_SUCCESS_WITH_INFO => SuccessWithInfo(Environment { handle: env }),
            SQLRETURN::SQL_ERROR => Error,
            _ => panic!("SQLAllocHandle returned an unexpected result"),
        }
    }

    pub fn set_odbc_version_3(&mut self) -> SetEnvAttrResult {

        use self::SetEnvAttrResult::*;

        match unsafe {
            SQLSetEnvAttr(self.handle,
                          SQL_ATTR_ODBC_VERSION,
                          SQL_OV_ODBC3 as *mut c_void,
                          0)
        } {
            SQLRETURN::SQL_SUCCESS => Success,
            SQLRETURN::SQL_SUCCESS_WITH_INFO => SuccessWithInfo,
            SQLRETURN::SQL_ERROR => Error,
            _ => panic!("SQLSetEnvAttr returned an unexpected result"),
        }
    }

    /// Iterates over data sources
    ///
    /// # Arguments
    /// * `direction` - Start with SQL_FETCH_FIRST to retrieve all data sources,
    ///   SQL_FETCH_FIRST_USER for user data sources and SQL_FETCH_FIRST_SYSTEM for system data
    ///   sources. In each case use SQL_FETCH_NEXT to retrieve the next element
    /// * `server_name` - To be filled with the server name
    /// * `description` - To be filled with the description
    ///
    /// # Result
    /// The tuple contains the required length of the buffers minus null-termination character.
    pub fn data_sources(&mut self,
                        direction: u16,
                        server_name: &mut [u8],
                        description: &mut [u8])
                        -> IterationResult<(i16, i16)> {
        self.impl_data_sources(SQLDataSources, direction, server_name, description)
    }

    pub fn drivers(&mut self,
                   direction: u16,
                   description: &mut [u8],
                   attributes: &mut [u8])
                   -> IterationResult<(i16, i16)> {
        self.impl_data_sources(SQLDrivers, direction, description, attributes)
    }

    // this private method uses the fact that SQLDataSources and SQLDrivers share the same signature
    fn impl_data_sources(&mut self,
                         c_function: SqlInfoFunction,
                         direction: u16,
                         server_name: &mut [u8],
                         description: &mut [u8])
                         -> IterationResult<(i16, i16)> {
        let (mut server_name_length, mut description_length): (i16, i16) = (0, 0);
        match unsafe {
            c_function(self.handle,
                       direction,
                       server_name.as_mut_ptr(),
                       as_buffer_length(server_name.len()),
                       &mut server_name_length as *mut i16,
                       description.as_mut_ptr(),
                       as_buffer_length(description.len()),
                       &mut description_length as *mut i16)
        } {
            SQLRETURN::SQL_SUCCESS => {
                IterationResult::Success((server_name_length, description_length))
            }
            SQLRETURN::SQL_SUCCESS_WITH_INFO => {
                IterationResult::SuccessWithInfo((server_name_length, description_length))
            }
            SQLRETURN::SQL_ERROR => IterationResult::Error,
            SQLRETURN::SQL_NO_DATA => IterationResult::NoData,
            _ => panic!("SQL Data Sources returned unexpected result"),
        }
    }
}

/// Signature shared by `raw::SQLDrivers` and `raw::SQLDataSources`
type SqlInfoFunction = unsafe extern "C" fn(SQLHENV,
                                            SQLUSMALLINT,
                                            *mut SQLCHAR,
                                            SQLSMALLINT,
                                            *mut SQLSMALLINT,
                                            *mut SQLCHAR,
                                            SQLSMALLINT,
                                            *mut SQLSMALLINT)
                                            -> SQLRETURN;

#[must_use]
pub enum IterationResult<T> {
    Success(T),
    SuccessWithInfo(T),
    NoData,
    Error,
}

/// Returned if an Environment is allocated
#[must_use]
pub enum EnvAllocResult {
    /// Creation of the environment is a success
    Success(Environment),
    /// Successfully created environment with warnings
    SuccessWithInfo(Environment),
    /// Allocation failed
    Error,
}

/// Returned if an Environment is allocated
#[must_use]
pub enum SetEnvAttrResult {
    Success,
    SuccessWithInfo,
    Error,
}