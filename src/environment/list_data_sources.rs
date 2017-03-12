use super::{Environment, Result, GetDiagRec};
use super::super::{ffi, Raii, Handle, Return};
use ffi::{SQLDataSources, SQLDrivers, SQLRETURN, SQLHENV, SQLSMALLINT, SQLCHAR, FetchOrientation};
use std::collections::HashMap;
use std::str::from_utf8;
use std::cmp::max;
use std::ptr::null_mut;

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

type SqlInfoMethod = fn(&Raii<ffi::Env>, ffi::FetchOrientation, &mut [u8], &mut [u8])
                        -> Return<Option<(i16, i16)>>;

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

    /// Stores all data source server names and descriptions in a Vec
    pub fn data_sources(&self) -> Result<Vec<DataSourceInfo>> {
        self.data_sources_impl(ffi::SQL_FETCH_FIRST)
    }

    /// Stores all sytem data source server names and descriptions in a Vec
    pub fn system_data_sources(&self) -> Result<Vec<DataSourceInfo>> {
        self.data_sources_impl(ffi::SQL_FETCH_FIRST_SYSTEM)
    }

    /// Stores all user data source server names and descriptions in a Vec
    pub fn user_data_sources(&self) -> Result<Vec<DataSourceInfo>> {
        self.data_sources_impl(ffi::SQL_FETCH_FIRST_USER)
    }

    /// Use SQL_FETCH_FIRST, SQL_FETCH_FIRST_USER or SQL_FETCH_FIRST_SYSTEM, to get all, user or
    /// system data sources
    fn data_sources_impl(&self, direction: ffi::FetchOrientation) -> Result<Vec<DataSourceInfo>> {

        // alloc_info iterates ones over every datasource to obtain the requiered buffer sizes
        let (max_name, max_desc, num_sources) = self.alloc_info(Raii::data_sources, direction)?;

        let mut source_list = Vec::with_capacity(num_sources);
        let mut name_buffer: Vec<_> = (0..(max_name + 1)).map(|_| 0u8).collect();
        let mut description_buffer: Vec<_> = (0..(max_desc + 1)).map(|_| 0u8).collect();

        // Before we call SQLDataSources with SQL_FETCH_NEXT, we have to call it with either
        // SQL_FETCH_FIRST, SQL_FETCH_FIRST_USER or SQL_FETCH_FIRST_SYSTEM, to get all, user or
        // system data sources
        if let Some((name, desc)) =
            self.get_info(Raii::data_sources,
                          direction,
                          &mut name_buffer,
                          &mut description_buffer)? {
            source_list.push(DataSourceInfo {
                server_name: name.to_owned(),
                description: desc.to_owned(),
            })
        } else {
            return Ok(source_list);
        }

        while let Some((name, desc)) =
            self.get_info(Raii::data_sources,
                          ffi::SQL_FETCH_NEXT,
                          &mut name_buffer,
                          &mut description_buffer)? {
            source_list.push(DataSourceInfo {
                server_name: name.to_owned(),
                description: desc.to_owned(),
            })
        }
        Ok(source_list)
    }

    /// Calls either SQLDrivers or SQLDataSources with the two given buffers and parses the result
    /// into a `(&str,&str)`
    fn get_info<'a, 'b>(&self,
                        f: SqlInfoMethod,
                        direction: ffi::FetchOrientation,
                        buf1: &'a mut [u8],
                        buf2: &'b mut [u8])
                        -> Result<Option<(&'a str, &'b str)>> {

        let result = f(&self.raii, direction, buf1, buf2);
        match result.into_result(self)? {
            Some((len1, len2)) => {
                Ok(Some((from_utf8(&buf1[0..(len1 as usize)]).unwrap(),
                         from_utf8(&buf2[0..(len2 as usize)]).unwrap())))
            }
            None => Ok(None),
        }
    }

    /// Finds the maximum size required for description buffers
    fn alloc_info(&self,
                  f: SqlInfoMethod,
                  direction: ffi::FetchOrientation)
                  -> Result<(ffi::SQLSMALLINT, ffi::SQLSMALLINT, usize)> {
        let mut string_buf1 = [0; 0];
        let mut string_buf2 = [0; 0];
        let mut max1 = 0;
        let mut max2 = 0;
        let mut count = 0;

        let mut result = f(&self.raii, direction, &mut string_buf1, &mut string_buf2);

        loop {

            match result {
                Return::Success(Some((buf1_length_out, buf2_length_out))) |
                Return::SuccessWithInfo(Some((buf1_length_out, buf2_length_out))) => {
                    count += 1;
                    max1 = max(max1, buf1_length_out);
                    max2 = max(max2, buf2_length_out);
                }
                Return::Success(None) |
                Return::SuccessWithInfo(None) => break,
                Return::Error => {
                    let diag = self.get_diag_rec(1).unwrap();
                    error!("{}", diag);
                    return Err(diag);
                }
            }

            result = f(&self.raii,
                       ffi::SQL_FETCH_NEXT,
                       &mut string_buf1,
                       &mut string_buf2)
        }

        Ok((max1, max2, count))
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
    fn data_sources(&self,
                    direction: FetchOrientation,
                    server_name: &mut [u8],
                    description: &mut [u8])
                    -> Return<Option<(i16, i16)>> {
        self.impl_data_sources(SQLDataSources, direction, server_name, description)
    }

    fn drivers(&self,
               direction: FetchOrientation,
               description: &mut [u8],
               attributes: &mut [u8])
               -> Return<Option<(i16, i16)>> {
        self.impl_data_sources(SQLDrivers, direction, description, attributes)
    }

    // this method uses the fact that SQLDataSources and SQLDrivers share the same signature
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

fn as_out_buffer(buffer: &mut [u8]) -> *mut u8 {
    if buffer.len() == 0 {
        null_mut()
    } else {
        buffer.as_mut_ptr()
    }
}

fn as_buffer_length(n: usize) -> ffi::SQLSMALLINT {
    use std;
    if n > std::i16::MAX as usize {
        std::i16::MAX
    } else {
        n as i16
    }
}

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
