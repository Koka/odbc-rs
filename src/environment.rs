//! This module implements the ODBC Environment
use super::{Error, DiagRec, Result, raw};
use std::collections::HashMap;
use std;

/// Handle to an ODBC Environment
///
/// Creating an instance of this type is the first thing you do then using ODBC. The environment
/// must outlive all connections created with it
pub struct Environment {
    handle: raw::SQLHENV,
}

/// Holds name and description of a datasource
///
/// Can be obtained via `Environment::data_sources`
#[derive(Clone, Debug)]
pub struct DataSourceInfo {
    /// Name of the data source
    pub server_name: String,
    /// Description of the data source
    pub description: String,
}

/// Struct holding information available on a driver.
///
/// Can be obtained via `Environment::drivers`
#[derive(Clone, Debug)]
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
        unsafe {
            let mut env = std::ptr::null_mut();
            let mut result =
                match raw::SQLAllocHandle(raw::SQL_HANDLE_ENV, std::ptr::null_mut(), &mut env) {
                    raw::SQL_SUCCESS => Environment { handle: env },
                    raw::SQL_SUCCESS_WITH_INFO => Environment { handle: env },
                    // Driver Manager failed to allocate environment
                    raw::SQL_ERROR => return Err(Error::EnvAllocFailure),
                    _ => unreachable!(),
                };
            // no leak if we return an error here, env handle is already wrapped and would be
            // dropped
            result.set_attribute(raw::SQL_ATTR_ODBC_VERSION,
                               raw::SQL_OV_ODBC3 as *mut std::os::raw::c_void,
                               0)?;

            Ok(result)
        }
    }

    /// Stores all driver description and attributes in a Vec
    pub fn drivers(&self) -> Result<Vec<DriverInfo>> {
        // Iterate twice, once for reading the maximum required buffer lengths so we can read
        // everything without truncating and a second time for actually storing the values
        let mut desc_length_out: raw::SQLSMALLINT = 0;
        let mut attr_length_out: raw::SQLSMALLINT = 0;
        let mut result;

        // alloc_info iterates ones over every driver to obtain the requiered buffer sizes
        let (max_desc, max_attr, num_drivers) = self.alloc_info(raw::SQLDrivers)?;

        let mut driver_list = Vec::with_capacity(num_drivers);
        loop {
            let mut description_buffer: Vec<_> = (0..(max_desc + 1)).map(|_| 0u8).collect();
            let mut attribute_buffer: Vec<_> = (0..(max_attr + 1)).map(|_| 0u8).collect();
            unsafe {
                result = raw::SQLDrivers(self.handle,
                                         // Its ok to use fetch next here, since we know
                                         // last state has been SQL_NO_DATA
                                         raw::SQL_FETCH_NEXT,
                                         &mut description_buffer[0] as *mut u8,
                                         max_desc + 1,
                                         &mut desc_length_out as *mut raw::SQLSMALLINT,
                                         &mut attribute_buffer[0] as *mut u8,
                                         max_attr + 1,
                                         &mut attr_length_out as *mut raw::SQLSMALLINT);
            }
            match result {
                raw::SQL_SUCCESS |
                raw::SQL_SUCCESS_WITH_INFO => {
                    description_buffer.resize(desc_length_out as usize, 0);
                    driver_list.push(DriverInfo {
                        description: String::from_utf8(description_buffer)
                            .expect("String returned by Driver Manager should be utf8 encoded"),
                        attributes: Self::parse_attributes(attribute_buffer),
                    })
                }
                raw::SQL_ERROR => unsafe {
                    return Err(Error::SqlError(DiagRec::create(raw::SQL_HANDLE_ENV, self.handle)));
                },
                raw::SQL_NO_DATA => break,
                /// The only other value allowed by ODBC here is SQL_INVALID_HANDLE. We protect the
                /// validity of this handle with our invariant. In save code the user should not be
                /// able to reach this code path.
                _ => panic!("Environment invariant violated"),
            }
        }
        Ok(driver_list)
    }

    /// Stores all data source server names and descriptions in a Vec
    pub fn data_sources(&self) -> Result<Vec<DataSourceInfo>> {
        // Iterate twice, once for reading the maximum required buffer lengths so we can read
        // everything without truncating and a second time for actually storing the values
        let mut name_length_out: raw::SQLSMALLINT = 0;
        let mut desc_length_out: raw::SQLSMALLINT = 0;
        let mut result;

        // alloc_info iterates ones over every datasource to obtain the requiered buffer sizes
        let (max_name, max_desc, num_sources) = self.alloc_info(raw::SQLDataSources)?;

        let mut source_list = Vec::with_capacity(num_sources);
        loop {
            let mut name_buffer: Vec<_> = (0..(max_name + 1)).map(|_| 0u8).collect();
            let mut description_buffer: Vec<_> = (0..(max_desc + 1)).map(|_| 0u8).collect();
            unsafe {
                result = raw::SQLDataSources(self.handle,
                                             // Its ok to use fetch next here, since we know
                                             // last state has been SQL_NO_DATA
                                             raw::SQL_FETCH_NEXT,
                                             &mut name_buffer[0] as *mut u8,
                                             max_name + 1,
                                             &mut name_length_out as *mut raw::SQLSMALLINT,
                                             &mut description_buffer[0] as *mut u8,
                                             max_desc + 1,
                                             &mut desc_length_out as *mut raw::SQLSMALLINT);
            }
            match result {
                raw::SQL_SUCCESS |
                raw::SQL_SUCCESS_WITH_INFO => {
                    name_buffer.resize(name_length_out as usize, 0);
                    description_buffer.resize(desc_length_out as usize, 0);
                    source_list.push(DataSourceInfo {
                        server_name: String::from_utf8(name_buffer)
                            .expect("String returned by Driver Manager should be utf8 encoded"),
                        description: String::from_utf8(description_buffer)
                            .expect("String returned by Driver Manager should be utf8 encoded"),
                    })
                }
                raw::SQL_ERROR => unsafe {
                    return Err(Error::SqlError(DiagRec::create(raw::SQL_HANDLE_ENV, self.handle)));
                },
                raw::SQL_NO_DATA => break,
                /// The only other value allowed by ODBC here is SQL_INVALID_HANDLE. We protect the
                /// validity of this handle with our invariant. In save code the user should not be
                /// able to reach this code path.
                _ => panic!("Environment invariant violated"),
            }
        }
        Ok(source_list)
    }

    /// Allows access to the raw ODBC handle
    pub unsafe fn raw(&mut self) -> raw::SQLHENV {
        self.handle
    }

    /// Allows setting attributes to Environment
    pub unsafe fn set_attribute(&mut self,
                                attribute: raw::SQLINTEGER,
                                value: raw::SQLPOINTER,
                                length: raw::SQLINTEGER)
                                -> Result<()> {
        match raw::SQLSetEnvAttr(self.handle, attribute, value, length) {
            raw::SQL_SUCCESS => Ok(()),
            raw::SQL_SUCCESS_WITH_INFO => Ok(()),
            _ => Err(Error::SqlError(DiagRec::create(raw::SQL_HANDLE_ENV, self.handle))),
        }
    }

    fn alloc_info(&self,
                  f: SqlInfoFunction)
                  -> Result<(raw::SQLSMALLINT, raw::SQLSMALLINT, usize)> {
        let string_buf = std::ptr::null_mut();
        let mut buf1_length_out: raw::SQLSMALLINT = 0;
        let mut buf2_length_out: raw::SQLSMALLINT = 0;
        let mut max1 = 0;
        let mut max2 = 0;
        let mut count = 0;
        let mut result = unsafe {
            // Although the rather lengthy function call kind of blows the code, let's do the first
            // one using SQL_FETCH_FIRST, so we list all drivers independent from environment state
            f(self.handle,
              raw::SQL_FETCH_FIRST,
              string_buf,
              0,
              &mut buf1_length_out as *mut raw::SQLSMALLINT,
              string_buf,
              0,
              &mut buf2_length_out as *mut raw::SQLSMALLINT)
        };
        loop {
            match result {
                raw::SQL_SUCCESS |
                raw::SQL_SUCCESS_WITH_INFO => {
                    count += 1;
                    max1 = std::cmp::max(max1, buf1_length_out);
                    max2 = std::cmp::max(max2, buf2_length_out);
                }
                raw::SQL_NO_DATA => break,
                raw::SQL_ERROR => unsafe {
                    return Err(Error::SqlError(DiagRec::create(raw::SQL_HANDLE_ENV, self.handle)));
                },
                /// The only other value allowed by ODBC here is SQL_INVALID_HANDLE. We protect the
                /// validity of this handle with our invariant. In save code the user should not be
                /// able to reach this code path.
                _ => panic!("Environment invariant violated"),
            }
            unsafe {
                result = f(self.handle,
                           raw::SQL_FETCH_NEXT,
                           string_buf,
                           0,
                           &mut buf1_length_out as *mut raw::SQLSMALLINT,
                           string_buf,
                           0,
                           &mut buf2_length_out as *mut raw::SQLSMALLINT);
            }
        }

        Ok((max1, max2, count))
    }

    /// Called by drivers to pares list of attributes
    ///
    /// Key value pairs are seperated by `\0`. Key and value are seperated by `=`
    fn parse_attributes(attribute_buffer: Vec<u8>) -> HashMap<String, String> {
        String::from_utf8(attribute_buffer)
            .expect("String returned by Driver Manager should be utf8 encoded")
            .split('\0')
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

impl Drop for Environment {
    fn drop(&mut self) {
        unsafe {
            raw::SQLFreeEnv(self.handle);
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn parse_attributes() {
        let buffer = "APILevel=2\0ConnectFunctions=YYY\0CPTimeout=60\0DriverODBCVer=03.\
                      50\0FileUsage=0\0SQLLevel=1\0UsageCount=1\0\0"
            .as_bytes()
            .iter()
            .cloned()
            .collect();
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