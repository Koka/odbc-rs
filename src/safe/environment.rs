use super::{as_buffer_length, as_out_buffer};
use super::super::{ffi, Raii, Handle, Return};
use ffi::{SQLSetEnvAttr, SQLDataSources, SQLDrivers, SQLRETURN, SQLHENV, SQLSMALLINT, SQLCHAR,
          SQL_ATTR_ODBC_VERSION, SQL_OV_ODBC3, FetchOrientation};
use std::os::raw::c_void;

impl Raii<ffi::Env> {
    pub fn set_odbc_version_3(&mut self) -> Return<()> {

        match unsafe {
                  SQLSetEnvAttr(self.handle(),
                                SQL_ATTR_ODBC_VERSION,
                                SQL_OV_ODBC3 as *mut c_void,
                                0)
              } {
            SQLRETURN::SQL_SUCCESS => Return::Success(()),
            SQLRETURN::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(()),
            SQLRETURN::SQL_ERROR => Return::Error,
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
                        direction: FetchOrientation,
                        server_name: &mut [u8],
                        description: &mut [u8])
                        -> IterationResult<(i16, i16)> {
        self.impl_data_sources(SQLDataSources, direction, server_name, description)
    }

    pub fn drivers(&mut self,
                   direction: FetchOrientation,
                   description: &mut [u8],
                   attributes: &mut [u8])
                   -> IterationResult<(i16, i16)> {
        self.impl_data_sources(SQLDrivers, direction, description, attributes)
    }

    // this private method uses the fact that SQLDataSources and SQLDrivers share the same signature
    fn impl_data_sources(&mut self,
                         c_function: SqlInfoFunction,
                         direction: FetchOrientation,
                         server_name: &mut [u8],
                         description: &mut [u8])
                         -> IterationResult<(i16, i16)> {
        let (mut server_name_length, mut description_length): (i16, i16) = (0, 0);
        match unsafe {
                  c_function(self.handle(),
                             direction,
                             as_out_buffer(server_name),
                             as_buffer_length(server_name.len()),
                             &mut server_name_length as *mut i16,
                             as_out_buffer(description),
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

/// Signature shared by `raw::SQLDrivers` and `ffi::SQLDataSources`
type SqlInfoFunction = unsafe extern "C" fn(SQLHENV,
                                            FetchOrientation,
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

