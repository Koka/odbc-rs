use super::{Error, Result, raw};
use std;

/// Handle to an ODBC Environment
///
/// Creating an instance of this type is the first thing you do then using ODBC. The environment
/// must outlive all connections created with it
pub struct Environment {
    handle: raw::SQLHENV,
}

/// Struct holding information available on a driver.
///
/// Can be obtained via `Environment::drivers`
#[derive(Clone, Debug)]
pub struct DriverInfo {
    pub description: String,
    pub attributes: Vec<(String, String)>,
}

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
                    raw::SQL_ERROR => return Err(Error {}),
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
        let string_buf = std::ptr::null_mut();
        let mut desc_length_out: raw::SQLSMALLINT = 0;
        let mut attr_length_out: raw::SQLSMALLINT = 0;
        let mut max_desc = 0;
        let mut max_attr = 0;
        let mut count = 0;
        let mut result;
        unsafe {
            // Although the rather lengthy function call kind of blows the code, let's do the first
            // one using SQL_FETCH_FIRST, so we list all drivers independent from environment state
            result = raw::SQLDrivers(self.handle,
                                     raw::SQL_FETCH_FIRST,
                                     string_buf,
                                     0,
                                     &mut desc_length_out as *mut raw::SQLSMALLINT,
                                     string_buf,
                                     0,
                                     &mut attr_length_out as *mut raw::SQLSMALLINT);
        }
        loop {
            match result {
                raw::SQL_SUCCESS |
                raw::SQL_SUCCESS_WITH_INFO => {
                    count += 1;
                    max_desc = std::cmp::max(max_desc, desc_length_out);
                    max_attr = std::cmp::max(max_attr, attr_length_out);
                }
                raw::SQL_NO_DATA => break,
                raw::SQL_ERROR => return Err(Error {}),
                _ => unreachable!(),
            }
            unsafe {
                result = raw::SQLDrivers(self.handle,
                                         raw::SQL_FETCH_NEXT,
                                         string_buf,
                                         0,
                                         &mut desc_length_out as *mut raw::SQLSMALLINT,
                                         string_buf,
                                         0,
                                         &mut attr_length_out as *mut raw::SQLSMALLINT);
            }
        }

        let mut driver_list = Vec::with_capacity(count);
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
                raw::SQL_ERROR => return Err(Error {}),
                raw::SQL_NO_DATA => break,
                _ => unreachable!(),
            }
        }
        Ok(driver_list)
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
            _ => Err(Error {}),
        }
    }

    /// Called by drivers to pares list of attributes
    ///
    /// Key value pairs are seperated by `\0`. Key and value are seperated by `=`
    fn parse_attributes(attribute_buffer: Vec<u8>) -> Vec<(String, String)> {
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
        let expected: Vec<_> = [("APILevel", "2"),
                                ("ConnectFunctions", "YYY"),
                                ("CPTimeout", "60"),
                                ("DriverODBCVer", "03.50"),
                                ("FileUsage", "0"),
                                ("SQLLevel", "1"),
                                ("UsageCount", "1")]
            .iter()
            .map(|&(k, v)| (k.to_string(), v.to_string()))
            .collect();
        assert_eq!(expected, attributes);
    }
}