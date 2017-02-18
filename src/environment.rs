//! This module implements the ODBC Environment
use super::{Error, Result, raw};
use super::raw::SQLRETURN::*;
use super::safe;
use safe::{Handle, GetDiagRec};
use std::collections::HashMap;
use std;

/// Handle to an ODBC Environment
///
/// Creating an instance of this type is the first thing you do then using ODBC. The environment
/// must outlive all connections created with it
pub struct Environment {
    handle: safe::Environment,
}

impl safe::GetDiagRec for Environment {
    fn get_diagnostic_record(&self, record_number: i16) -> Option<safe::DiagRec> {
        self.handle.get_diagnostic_record(record_number)
    }
}

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

/// Signature shared by `raw::SQLDrivers` and `raw::SQLDataSources`
type SqlInfoFunction = unsafe extern "C" fn(raw::SQLHENV,
                                            raw::SQLUSMALLINT,
                                            *mut raw::SQLCHAR,
                                            raw::SQLSMALLINT,
                                            *mut raw::SQLSMALLINT,
                                            *mut raw::SQLCHAR,
                                            raw::SQLSMALLINT,
                                            *mut raw::SQLSMALLINT)
                                            -> raw::SQLRETURN;

impl Environment {
    /// Allocates a new ODBC Environment
    ///
    /// Declares the Application's ODBC Version to be 3
    pub fn new() -> Result<Environment> {

        use safe::{EnvAllocResult, SetEnvAttrResult};

        let mut result = match safe::Environment::new() {
            EnvAllocResult::Success(env) => Environment { handle: env },
            EnvAllocResult::SuccessWithInfo(env) => Environment { handle: env },
            EnvAllocResult::Error => return Err(Error::EnvAllocFailure),
        };

        match result.handle.set_odbc_version_3() {
            SetEnvAttrResult::Success => Ok(result),
            SetEnvAttrResult::SuccessWithInfo => Ok(result),
            SetEnvAttrResult::Error => {
                Err(Error::SqlError(result.handle.get_diagnostic_record(1).unwrap()))
            }
        }
    }

    /// Stores all driver description and attributes in a Vec
    pub fn drivers(&self) -> Result<Vec<DriverInfo>> {
        // Iterate twice, once for reading the maximum required buffer lengths so we can read
        // everything without truncating and a second time for actually storing the values
        // alloc_info iterates ones over every driver to obtain the requiered buffer sizes
        let (max_desc, max_attr, num_drivers) =
            unsafe { self.alloc_info(raw::SQLDrivers, raw::SQL_FETCH_FIRST) }?;

        let mut driver_list = Vec::with_capacity(num_drivers);
        let mut description_buffer: Vec<_> = (0..(max_desc + 1)).map(|_| 0u8).collect();
        let mut attribute_buffer: Vec<_> = (0..(max_attr + 1)).map(|_| 0u8).collect();
        while let Some((desc, attr)) = unsafe {
            self.get_info(raw::SQLDrivers,
                          raw::SQL_FETCH_NEXT,
                          &mut description_buffer,
                          &mut attribute_buffer)
        }? {
            driver_list.push(DriverInfo {
                description: desc.to_owned(),
                attributes: Self::parse_attributes(attr),
            })
        }
        Ok(driver_list)
    }

    /// Stores all data source server names and descriptions in a Vec
    pub fn data_sources(&self) -> Result<Vec<DataSourceInfo>> {
        unsafe { self.data_sources_impl(raw::SQL_FETCH_FIRST) }
    }

    /// Stores all sytem data source server names and descriptions in a Vec
    pub fn system_data_sources(&self) -> Result<Vec<DataSourceInfo>> {
        unsafe { self.data_sources_impl(raw::SQL_FETCH_FIRST_SYSTEM) }
    }

    /// Stores all user data source server names and descriptions in a Vec
    pub fn user_data_sources(&self) -> Result<Vec<DataSourceInfo>> {
        unsafe { self.data_sources_impl(raw::SQL_FETCH_FIRST_USER) }
    }

    /// Use SQL_FETCH_FIRST, SQL_FETCH_FIRST_USER or SQL_FETCH_FIRST_SYSTEM, to get all, user or
    /// system data sources
    unsafe fn data_sources_impl(&self,
                                direction: raw::SQLUSMALLINT)
                                -> Result<Vec<DataSourceInfo>> {

        // alloc_info iterates ones over every datasource to obtain the requiered buffer sizes
        let (max_name, max_desc, num_sources) = self.alloc_info(raw::SQLDataSources, direction)?;

        let mut source_list = Vec::with_capacity(num_sources);
        let mut name_buffer: Vec<_> = (0..(max_name + 1)).map(|_| 0u8).collect();
        let mut description_buffer: Vec<_> = (0..(max_desc + 1)).map(|_| 0u8).collect();

        // Before we call SQLDataSources with SQL_FETCH_NEXT, we have to call it with either
        // SQL_FETCH_FIRST, SQL_FETCH_FIRST_USER or SQL_FETCH_FIRST_SYSTEM, to get all, user or
        // system data sources
        if let Some((name, desc)) = self.get_info(raw::SQLDataSources,
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

        while let Some((name, desc)) = self.get_info(raw::SQLDataSources,
                      raw::SQL_FETCH_NEXT,
                      &mut name_buffer,
                      &mut description_buffer)? {
            source_list.push(DataSourceInfo {
                server_name: name.to_owned(),
                description: desc.to_owned(),
            })
        }
        Ok(source_list)
    }

    /// Allows access to the raw ODBC handle
    pub unsafe fn raw(&mut self) -> raw::SQLHENV {
        self.handle.handle()
    }

    /// Calls either SQLDrivers or SQLDataSources with the two given buffers and parses the result
    /// into a `(&str,&str)`
    unsafe fn get_info<'a, 'b>(&self,
                               f: SqlInfoFunction,
                               direction: raw::SQLUSMALLINT,
                               buf1: &'a mut [u8],
                               buf2: &'b mut [u8])
                               -> Result<Option<(&'a str, &'b str)>> {
        let mut len1: raw::SQLSMALLINT = 0;
        let mut len2: raw::SQLSMALLINT = 0;

        let result = f(self.handle.handle(),
                       // Its ok to use fetch next here, since we know
                       // last state has been SQL_NO_DATA
                       direction,
                       &mut buf1[0] as *mut u8,
                       buf1.len() as raw::SQLSMALLINT,
                       &mut len1 as *mut raw::SQLSMALLINT,
                       &mut buf2[0] as *mut u8,
                       buf2.len() as raw::SQLSMALLINT,
                       &mut len2 as *mut raw::SQLSMALLINT);
        match result {
            SQL_SUCCESS |
            SQL_SUCCESS_WITH_INFO => {
                Ok(Some((std::str::from_utf8(&buf1[0..(len1 as usize)]).unwrap(),
                         std::str::from_utf8(&buf2[0..(len2 as usize)]).unwrap())))
            }
            SQL_ERROR => Err(Error::SqlError(self.handle.get_diagnostic_record(1).unwrap())),
            SQL_NO_DATA => Ok(None),
            /// The only other value allowed by ODBC here is SQL_INVALID_HANDLE. We protect the
            /// validity of this handle with our invariant. In save code the user should not be
            /// able to reach this code path.
            _ => panic!("Environment invariant violated"),
        }
    }

    /// Finds the maximum size required for description buffers
    unsafe fn alloc_info(&self,
                         f: SqlInfoFunction,
                         direction: raw::SQLUSMALLINT)
                         -> Result<(raw::SQLSMALLINT, raw::SQLSMALLINT, usize)> {
        let string_buf = std::ptr::null_mut();
        let mut buf1_length_out: raw::SQLSMALLINT = 0;
        let mut buf2_length_out: raw::SQLSMALLINT = 0;
        let mut max1 = 0;
        let mut max2 = 0;
        let mut count = 0;
        let mut result = f(self.handle.handle(),
                           direction,
                           string_buf,
                           0,
                           &mut buf1_length_out as *mut raw::SQLSMALLINT,
                           string_buf,
                           0,
                           &mut buf2_length_out as *mut raw::SQLSMALLINT);
        loop {
            match result {
                SQL_SUCCESS |
                SQL_SUCCESS_WITH_INFO => {
                    count += 1;
                    max1 = std::cmp::max(max1, buf1_length_out);
                    max2 = std::cmp::max(max2, buf2_length_out);
                }
                SQL_NO_DATA => break,
                SQL_ERROR => {
                    return Err(Error::SqlError(self.handle.get_diagnostic_record(1).unwrap()));
                }
                /// The only other value allowed by ODBC here is SQL_INVALID_HANDLE. We protect the
                /// validity of this handle with our invariant. In save code the user should not be
                /// able to reach this code path.
                _ => panic!("Environment invariant violated"),
            }

            result = f(self.handle.handle(),
                       raw::SQL_FETCH_NEXT,
                       string_buf,
                       0,
                       &mut buf1_length_out as *mut raw::SQLSMALLINT,
                       string_buf,
                       0,
                       &mut buf2_length_out as *mut raw::SQLSMALLINT);
        }

        Ok((max1, max2, count))
    }

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
