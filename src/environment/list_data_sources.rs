use super::{Env, sys, safe, GenericError};

use std::error::Error;
use std::collections::HashMap;
use std::cmp::max;
use std::str::from_utf8;

/// Holds name and description of a datasource
///
/// Can be obtained via `Environment::data_sources`
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataSourceInfo {
    /// Name of the data source
    pub server_name: String,
    /// Description of the data source
    pub driver: String,
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

type SqlInfoMethod = fn(&mut safe::Environment<safe::Odbc3>,
                        sys::FetchOrientation,
                        &mut [u8],
                        &mut [u8])
                        -> safe::ReturnOption<(sys::SQLSMALLINT, sys::SQLSMALLINT)>;

impl Env {
    /// Stores all driver description and attributes in a Vec
    pub fn drivers() -> Result<Vec<DriverInfo>, Box<Error>> {
        Env::with_env_mut(|env| {

            // Iterate twice, once for reading the maximum required buffer lengths so we can read
            // everything without truncating and a second time for actually storing the values
            // alloc_info iterates ones over every driver to obtain the requiered buffer sizes
            let (max_desc, max_attr, num_drivers) = Env::alloc_info(
                env,
                safe::Environment::drivers,
                sys::SQL_FETCH_FIRST,
            )?;

            let mut driver_list = Vec::with_capacity(num_drivers);
            let mut description_buffer = vec![0; (max_desc + 1) as usize];
            let mut attribute_buffer = vec![0; (max_attr + 1) as usize];
            while let Some((desc, attr)) =
            Env::get_info(
                env,
                safe::Environment::drivers,
                sys::SQL_FETCH_NEXT,
                &mut description_buffer,
                &mut attribute_buffer,
            )?
                {
                    driver_list.push(DriverInfo {
                        description: desc.to_owned(),
                        attributes: Self::parse_attributes(attr),
                    })
                }
            Ok(driver_list)
        })
    }

    /// Stores all data source server names and descriptions in a Vec
    pub fn data_sources() -> Result<Vec<DataSourceInfo>, Box<Error>> {
        Env::with_env_mut(|env| Env::data_sources_impl(env, sys::SQL_FETCH_FIRST))
    }

    /// Stores all system data source server names and descriptions in a Vec
    pub fn system_data_sources() -> Result<Vec<DataSourceInfo>, Box<Error>> {
        Env::with_env_mut(|env| Env::data_sources_impl(env, sys::SQL_FETCH_FIRST_SYSTEM))
    }

    /// Stores all user data source server names and descriptions in a Vec
    pub fn user_data_sources() -> Result<Vec<DataSourceInfo>, Box<Error>> {
        Env::with_env_mut(|env| Env::data_sources_impl(env, sys::SQL_FETCH_FIRST_USER))
    }

    /// Called by drivers to pares list of attributes
    ///
    /// Key value pairs are separated by `\0`. Key and value are seperated by `=`
    fn parse_attributes(attributes: &str) -> HashMap<String, String> {
        attributes
            .split('\0')
            .take_while(|kv_str| *kv_str != String::new())
            .map(|kv_str| {
                let mut iter = kv_str.split('=');
                let key = iter.next().map( |k| k.to_string() ).unwrap_or("Unknown".to_string());
                let value = iter.next().map( |v| v.to_string() ).unwrap_or("".to_string());
                (key, value)
            })
            .collect()
    }

    /// Use SQL_FETCH_FIRST, SQL_FETCH_FIRST_USER or SQL_FETCH_FIRST_SYSTEM, to get all, user or
    /// system data sources
    fn data_sources_impl(
        env: &mut safe::Environment<safe::Odbc3>,
        direction: sys::FetchOrientation,
    ) -> Result<Vec<DataSourceInfo>, Box<Error>> {

        // alloc_info iterates ones over every datasource to obtain the requiered buffer sizes
        let (max_name, max_desc, num_sources) =
            Env::alloc_info(env, safe::Environment::data_sources, direction)?;

        let mut source_list = Vec::with_capacity(num_sources);
        let mut name_buffer: Vec<_> = (0..(max_name + 1)).map(|_| 0u8).collect();
        let mut description_buffer: Vec<_> = (0..(max_desc + 1)).map(|_| 0u8).collect();

        // Before we call SQLDataSources with SQL_FETCH_NEXT, we have to call it with either
        // SQL_FETCH_FIRST, SQL_FETCH_FIRST_USER or SQL_FETCH_FIRST_SYSTEM, to get all, user or
        // system data sources
        if let Some((name, desc)) =
        Env::get_info(
            env,
            safe::Environment::data_sources,
            direction,
            &mut name_buffer,
            &mut description_buffer,
        )?
            {
                source_list.push(DataSourceInfo {
                    server_name: name.to_owned(),
                    driver: desc.to_owned(),
                })
            } else {
            return Ok(source_list);
        }

        while let Some((name, desc)) =
        Env::get_info(
            env,
            safe::Environment::data_sources,
            sys::SQL_FETCH_NEXT,
            &mut name_buffer,
            &mut description_buffer,
        )?
            {
                source_list.push(DataSourceInfo {
                    server_name: name.to_owned(),
                    driver: desc.to_owned(),
                })
            }
        Ok(source_list)
    }

    /// Finds the maximum size required for description buffers
    fn alloc_info(
        mut env: &mut safe::Environment<safe::Odbc3>,
        f: SqlInfoMethod,
        direction: sys::FetchOrientation,
    ) -> Result<(sys::SQLSMALLINT, sys::SQLSMALLINT, usize), impl Error> {
        let mut string_buf1 = [0; 0];
        let mut string_buf2 = [0; 0];
        let mut max1 = 0;
        let mut max2 = 0;
        let mut count = 0;

        let mut result = f(
            env,
            direction,
            &mut string_buf1,
            &mut string_buf2,
        );

        loop {
            match result {
                safe::ReturnOption::Success((buf1_length_out, buf2_length_out)) |
                safe::ReturnOption::Info((buf1_length_out, buf2_length_out)) => {
                    count += 1;
                    max1 = max(max1, buf1_length_out);
                    max2 = max(max2, buf2_length_out);
                }
                safe::ReturnOption::NoData(()) => break,
                safe::ReturnOption::Error(()) => {
                    return Err(GenericError("Error getting ODBC metainformation".to_owned()));
                }
            }

            result = f(
                &mut env,
                sys::SQL_FETCH_NEXT,
                &mut string_buf1,
                &mut string_buf2,
            )
        }

        Ok((max1, max2, count))
    }

    /// Calls either SQLDrivers or SQLDataSources with the two given buffers and parses the result
    /// into a `(&str,&str)`
    fn get_info<'a, 'b>(
        env: &mut safe::Environment<safe::Odbc3>,
        f: SqlInfoMethod,
        direction: sys::FetchOrientation,
        buf1: &'a mut [u8],
        buf2: &'b mut [u8],
    ) -> Result<Option<(&'a str, &'b str)>, Box<Error>> {
        let result = f(env, direction, buf1, buf2);

        match result {
            safe::ReturnOption::Success((len1, len2)) | safe::ReturnOption::Info((len1, len2)) => {
                let a = from_utf8(&buf1[0..(len1 as usize)])?;
                let b = from_utf8(&buf2[0..(len2 as usize)])?;
                Ok(Some((a, b)))
            },
            safe::ReturnOption::NoData(_) => Ok(None),
            safe::ReturnOption::Error(_) => Err(Box::new(GenericError("Error getting ODBC metainformation".to_owned())))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_attributes() {
        let buffer = "APILevel=2\0ConnectFunctions=YYY\0CPTimeout=60\0DriverODBCVer=03.\
                      50\0FileUsage=0\0OOPS=\0SQLLevel=1\0UsageCount=1\0\0";
        let attributes = Env::parse_attributes(buffer);
        assert_eq!(attributes["APILevel"], "2");
        assert_eq!(attributes["ConnectFunctions"], "YYY");
        assert_eq!(attributes["CPTimeout"], "60");
        assert_eq!(attributes["DriverODBCVer"], "03.50");
        assert_eq!(attributes["FileUsage"], "0");
        assert_eq!(attributes["OOPS"], "");
        assert_eq!(attributes["SQLLevel"], "1");
        assert_eq!(attributes["UsageCount"], "1");
    }
}