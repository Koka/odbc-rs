//! Implements the ODBC Environment
mod set_version;
mod list_data_sources;
pub use self::list_data_sources::{DataSourceInfo,DriverInfo};
use super::{Result, Return, ffi, GetDiagRec, Raii, Handle, EnvAllocError};
use std;
/// Handle to an ODBC Environment
///
/// Creating an instance of this type is the first thing you do then using ODBC. The environment
/// must outlive all connections created with it
pub struct Environment {
    raii: Raii<ffi::Env>,
}

impl Handle for Environment {
    type To = ffi::Env;
    unsafe fn handle(&self) -> ffi::SQLHENV {
        self.raii.handle()
    }
}

type SqlInfoMethod = fn(&Raii<ffi::Env>,
                        ffi::FetchOrientation,
                        &mut [u8],
                        &mut [u8])
                        -> Return<Option<(i16, i16)>>;

impl Environment {
    /// Allocates a new ODBC Environment
    ///
    /// Declares the Application's ODBC Version to be 3
    pub fn new() -> std::result::Result<Environment, EnvAllocError> {

        match unsafe { Raii::new() } {
            Return::Success(env) |
            Return::SuccessWithInfo(env) => Ok(Environment { raii: env }),
            Return::Error => Err(EnvAllocError),
        }
    }

    /// Tells the driver(s) that we will use features of up to ODBC version 3
    pub fn set_odbc_version_3(&mut self) -> Result<()> {
        match self.raii.set_odbc_version_3() {
            Return::Success(()) |
            Return::SuccessWithInfo(()) => Ok(()),
            Return::Error => Err(self.get_diag_rec(1).unwrap()),
        }
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
        match result {
            Return::Success(Some((len1, len2))) |
            Return::SuccessWithInfo(Some((len1, len2))) => {
                Ok(Some((std::str::from_utf8(&buf1[0..(len1 as usize)]).unwrap(),
                         std::str::from_utf8(&buf2[0..(len2 as usize)]).unwrap())))
            }
            Return::Error => Err(self.raii.get_diag_rec(1).unwrap()),
            Return::Success(None) | Return::SuccessWithInfo(None) => Ok(None),
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

        let mut result = f(&self.raii,
                           direction,
                           &mut string_buf1,
                           &mut string_buf2);

        loop {
            match result {
                Return::Success(Some((buf1_length_out, buf2_length_out))) |
                Return::SuccessWithInfo(Some((buf1_length_out, buf2_length_out))) => {
                    count += 1;
                    max1 = std::cmp::max(max1, buf1_length_out);
                    max2 = std::cmp::max(max2, buf2_length_out);
                }
                Return::Success(None) | Return::SuccessWithInfo(None)=> break,
                Return::Error => {
                    return Err(self.raii
                                   .get_diag_rec(1)
                                   .unwrap());
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