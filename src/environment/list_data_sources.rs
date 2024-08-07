use super::{safe, try_into_option, DiagnosticRecord, Environment, GetDiagRec, Result, Version3};
use ffi;
use std::cmp::max;
use std::collections::HashMap;

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

type SqlInfoMethod = fn(
    &mut safe::Environment<safe::Odbc3>,
    ffi::FetchOrientation,
    &mut [u8],
    &mut [u8],
) -> safe::ReturnOption<(i16, i16)>;

impl Environment<Version3> {
    /// Called by drivers to pares list of attributes
    ///
    /// Key value pairs are separated by `\0`. Key and value are separated by `=`
    fn parse_attributes(attributes: &str) -> HashMap<String, String> {
        attributes
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

    /// Stores all driver description and attributes in a Vec
    pub fn drivers(&mut self) -> Result<Vec<DriverInfo>> {
        // Iterate twice, once for reading the maximum required buffer lengths so we can read
        // everything without truncating and a second time for actually storing the values
        // alloc_info iterates once over every driver to obtain the required buffer sizes
        let (max_desc, max_attr, num_drivers) =
            self.alloc_info(safe::Environment::drivers, ffi::SQL_FETCH_FIRST)?;

        let mut driver_list = Vec::with_capacity(num_drivers);

        if num_drivers > 0 {
            let mut description_buffer = vec![0; (max_desc + 1) as usize];
            let mut attribute_buffer = vec![0; (max_attr + 1) as usize];
            while let Some((desc, attr)) = self.get_info(
                safe::Environment::drivers,
                ffi::SQL_FETCH_NEXT,
                &mut description_buffer,
                &mut attribute_buffer,
            )? {
                driver_list.push(DriverInfo {
                    description: desc.into_owned(),
                    attributes: Self::parse_attributes(&attr),
                })
            }
        }

        Ok(driver_list)
    }

    /// Stores all data source server names and descriptions in a Vec
    pub fn data_sources(&mut self) -> Result<Vec<DataSourceInfo>> {
        self.data_sources_impl(ffi::SQL_FETCH_FIRST)
    }

    /// Stores all system data source server names and descriptions in a Vec
    pub fn system_data_sources(&mut self) -> Result<Vec<DataSourceInfo>> {
        self.data_sources_impl(ffi::SQL_FETCH_FIRST_SYSTEM)
    }

    /// Stores all user data source server names and descriptions in a Vec
    pub fn user_data_sources(&mut self) -> Result<Vec<DataSourceInfo>> {
        self.data_sources_impl(ffi::SQL_FETCH_FIRST_USER)
    }

    /// Use SQL_FETCH_FIRST, SQL_FETCH_FIRST_USER or SQL_FETCH_FIRST_SYSTEM, to get all, user or
    /// system data sources
    fn data_sources_impl(
        &mut self,
        direction: ffi::FetchOrientation,
    ) -> Result<Vec<DataSourceInfo>> {
        // alloc_info iterates once over every datasource to obtain the required buffer sizes
        let (max_name, max_desc, num_sources) =
            self.alloc_info(safe::Environment::data_sources, direction)?;

        let mut source_list = Vec::with_capacity(num_sources);

        if num_sources > 0 {
            let mut name_buffer: Vec<_> = (0..(max_name + 1)).map(|_| 0u8).collect();
            let mut description_buffer: Vec<_> = (0..(max_desc + 1)).map(|_| 0u8).collect();

            // Before we call SQLDataSources with SQL_FETCH_NEXT, we have to call it with either
            // SQL_FETCH_FIRST, SQL_FETCH_FIRST_USER or SQL_FETCH_FIRST_SYSTEM, to get all, user or
            // system data sources
            if let Some((name, desc)) = self.get_info(
                safe::Environment::data_sources,
                direction,
                &mut name_buffer,
                &mut description_buffer,
            )? {
                source_list.push(DataSourceInfo {
                    server_name: name.into_owned(),
                    driver: desc.into_owned(),
                })
            } else {
                return Ok(source_list);
            }

            while let Some((name, desc)) = self.get_info(
                safe::Environment::data_sources,
                ffi::SQL_FETCH_NEXT,
                &mut name_buffer,
                &mut description_buffer,
            )? {
                source_list.push(DataSourceInfo {
                    server_name: name.into_owned(),
                    driver: desc.into_owned(),
                })
            }
        }

        Ok(source_list)
    }

    /// Calls either SQLDrivers or SQLDataSources with the two given buffers and parses the result
    /// into a `(&str,&str)`
    fn get_info<'a, 'b>(
        &mut self,
        f: SqlInfoMethod,
        direction: ffi::FetchOrientation,
        buf1: &'a mut [u8],
        buf2: &'b mut [u8],
    ) -> Result<Option<(::std::borrow::Cow<'a, str>, ::std::borrow::Cow<'b, str>)>> {
        let result = f(&mut self.safe, direction, buf1, buf2);
        match try_into_option(result, self)? {
            Some((len1, len2)) => unsafe {
                Ok(Some((
                    ::environment::DB_ENCODING
                        .decode(&buf1[0..(len1 as usize)])
                        .0,
                    ::environment::DB_ENCODING
                        .decode(&buf2[0..(len2 as usize)])
                        .0,
                )))
            },
            None => Ok(None),
        }
    }

    /// Finds the maximum size required for description buffers
    fn alloc_info(
        &mut self,
        f: SqlInfoMethod,
        direction: ffi::FetchOrientation,
    ) -> Result<(ffi::SQLSMALLINT, ffi::SQLSMALLINT, usize)> {
        // In theory, we should use zero-length buffers here
        // However, if we do SQLDrivers gives us 0-length values for
        // the attributes length, which is incorrect, so we allocated a
        // reasonably large array which seems to make it do the right thing
        let mut string_buf1 = [0; 1024];
        let mut string_buf2 = [0; 1024];

        let mut max1 = 0;
        let mut max2 = 0;
        let mut count = 0;

        let mut result = f(
            &mut self.safe,
            direction,
            &mut string_buf1,
            &mut string_buf2,
        );

        loop {
            match result {
                safe::ReturnOption::Success((buf1_length_out, buf2_length_out))
                | safe::ReturnOption::Info((buf1_length_out, buf2_length_out)) => {
                    count += 1;
                    max1 = max(max1, buf1_length_out);
                    max2 = max(max2, buf2_length_out);
                }
                safe::ReturnOption::NoData(()) => break,
                safe::ReturnOption::Error(()) => {
                    let diag = self.get_diag_rec(1).unwrap_or_else(DiagnosticRecord::empty);
                    error!("{}", diag);
                    return Err(diag);
                }
            }

            result = f(
                &mut self.safe,
                ffi::SQL_FETCH_NEXT,
                &mut string_buf1,
                &mut string_buf2,
            )
        }

        Ok((max1, max2, count))
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
