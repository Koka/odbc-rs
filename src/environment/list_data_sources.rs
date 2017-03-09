use super::{Environment, Result};
use super::super::{ffi, Raii, Handle, Return, as_buffer_length, as_out_buffer};
use ffi::{SQLDataSources, SQLDrivers, SQLRETURN, SQLHENV, SQLSMALLINT, SQLCHAR, FetchOrientation};
use std::collections::HashMap;

/// Holds name and description of a datasource
///
/// Can be obtained via `Environment::data_sources`
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataSourceInfo {
    /// Name of the data source
    pub server_name: String,
    /// Description of the data source
    pub description: String,
}

/// Struct holding information available on a driver.
///
/// Can be obtained via `Environment::drivers`
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DriverInfo {
    /// Name of the odbc driver
    pub description: String,
    /// List of attributes of the odbc driver
    pub attributes: HashMap<String, String>,
}

impl Environment {
    /// Called by drivers to pares list of attributes
    ///
    /// Key value pairs are seperated by `\0`. Key and value are seperated by `=`
    fn parse_attributes(attributes: &str) -> HashMap<String, String> {
        attributes.split('\0')
            .take_while(|kv_str| *kv_str != String::new())
            .map(|kv_str| {
                     let mut iter = kv_str.split('=');
                     let key = iter.next().unwrap();
                     let value = iter.next().unwrap();
                     (key.to_string(), value.to_string())
                 })
            .collect()
    }

    /// Stores all driver description and attributes in a Vec
    pub fn drivers(&self) -> Result<Vec<DriverInfo>> {
        // Iterate twice, once for reading the maximum required buffer lengths so we can read
        // everything without truncating and a second time for actually storing the values
        // alloc_info iterates ones over every driver to obtain the requiered buffer sizes
        let (max_desc, max_attr, num_drivers) =
            self.alloc_info(Raii::drivers, ffi::SQL_FETCH_FIRST)?;

        let mut driver_list = Vec::with_capacity(num_drivers);
        let mut description_buffer = vec![0; (max_desc + 1) as usize];
        let mut attribute_buffer = vec![0; (max_attr + 1) as usize];
        while let Some((desc, attr)) =
            self.get_info(Raii::drivers,
                          ffi::SQL_FETCH_NEXT,
                          &mut description_buffer,
                          &mut attribute_buffer)? {
            driver_list.push(DriverInfo {
                                 description: desc.to_owned(),
                                 attributes: Self::parse_attributes(attr),
                             })
        }
        Ok(driver_list)
    }
}

impl Raii<ffi::Env> {
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
    pub fn data_sources(&self,
                        direction: FetchOrientation,
                        server_name: &mut [u8],
                        description: &mut [u8])
                        -> Return<Option<(i16, i16)>> {
        self.impl_data_sources(SQLDataSources, direction, server_name, description)
    }

    pub fn drivers(&self,
                   direction: FetchOrientation,
                   description: &mut [u8],
                   attributes: &mut [u8])
                   -> Return<Option<(i16, i16)>> {
        self.impl_data_sources(SQLDrivers, direction, description, attributes)
    }

    // this private method uses the fact that SQLDataSources and SQLDrivers share the same signature
    fn impl_data_sources(&self,
                         c_function: SqlInfoFunction,
                         direction: FetchOrientation,
                         server_name: &mut [u8],
                         description: &mut [u8])
                         -> Return<Option<(i16, i16)>> {
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
                Return::Success(Some((server_name_length, description_length)))
            }
            SQLRETURN::SQL_SUCCESS_WITH_INFO => {
                Return::SuccessWithInfo(Some((server_name_length, description_length)))
            }
            SQLRETURN::SQL_ERROR => Return::Error,
            SQLRETURN::SQL_NO_DATA => Return::Success(None),
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

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn parse_attributes() {
        let buffer = "APILevel=2\0ConnectFunctions=YYY\0CPTimeout=60\0DriverODBCVer=03.\
                      50\0FileUsage=0\0SQLLevel=1\0UsageCount=1\0\0";
        let attributes = Environment::parse_attributes(buffer);
        assert_eq!(attributes["APILevel"], "2");
        assert_eq!(attributes["ConnectFunctions"], "YYY");
        assert_eq!(attributes["CPTimeout"], "60");
        assert_eq!(attributes["DriverODBCVer"], "03.50");
        assert_eq!(attributes["FileUsage"], "0");
        assert_eq!(attributes["SQLLevel"], "1");
        assert_eq!(attributes["UsageCount"], "1");
    }
}

