//! Holds implementation of odbc connection
use super::{ffi, Environment, Return, Result, Raii, Error, GetDiagRec, Handle};
use super::ffi::SQLRETURN::*;
use std;
use std::marker::PhantomData;

/// Represents a connection to an ODBC data source
pub struct DataSource<'a> {
    raii: Raii<ffi::Dbc>,
    // we use phantom data to tell the borrow checker that we need to keep the environment alive for
    // the lifetime of the connection
    parent: PhantomData<&'a Environment>,
}

impl<'a> Handle for DataSource<'a> {
    type To = ffi::Dbc;
    unsafe fn handle(&self) -> ffi::SQLHDBC {
        self.raii.handle()
    }
}

impl<'a> DataSource<'a> {
    /// Connects to an ODBC data source
    ///
    /// # Arguments
    /// * `env` - Environment used to allocate the data source handle.
    /// * `dsn` - Data source name configured in the `odbc.ini` file
    /// * `usr` - User identifier
    /// * `pwd` - Authentication (usually password)
    pub fn with_dsn_and_credentials<'b>(env: &'b mut Environment,
                                        dsn: &str,
                                        usr: &str,
                                        pwd: &str)
                                        -> Result<DataSource<'b>> {
        let raii = match Raii::with_parent(env) {
            Return::Success(dbc) => dbc,
            Return::SuccessWithInfo(dbc) => dbc,
            Return::Error => return Err(Error::SqlError(env.get_diag_rec(1).unwrap())),
        };

        let data_source = DataSource {
            raii: raii,
            parent: PhantomData,
        };

        unsafe {
            match ffi::SQLConnect(data_source.handle(),
                                  dsn.as_ptr(),
                                  dsn.as_bytes().len() as ffi::SQLSMALLINT,
                                  usr.as_ptr(),
                                  usr.as_bytes().len() as ffi::SQLSMALLINT,
                                  pwd.as_ptr(),
                                  pwd.as_bytes().len() as ffi::SQLSMALLINT) {
                SQL_SUCCESS |
                SQL_SUCCESS_WITH_INFO => Ok(data_source),
                _ => Err(Error::SqlError(data_source.get_diag_rec(1).unwrap())),
            }
        }
    }

    /// `true` if the data source is set to READ ONLY mode, `false` otherwise.
    ///
    /// This characteristic pertains only to the data source itself; it is not characteristic of
    /// the driver that enables access to the data source. A driver that is read/write can be used
    /// with a data source that is read-only. If a driver is read-only, all of its data sources
    /// must be read-only.
    pub fn read_only(&self) -> Result<bool> {
        let mut buffer: [u8; 2] = [0; 2];

        unsafe {
            match ffi::SQLGetInfo(self.raii.handle(),
                                  ffi::SQL_DATA_SOURCE_READ_ONLY,
                                  buffer.as_mut_ptr() as *mut std::os::raw::c_void,
                                  buffer.len() as ffi::SQLSMALLINT,
                                  std::ptr::null_mut()) {
                SQL_SUCCESS |
                SQL_SUCCESS_WITH_INFO => {
                    Ok({
                           assert!(buffer[1] == 0);
                           match buffer[0] as char {
                               'N' => false,
                               'Y' => true,
                               _ => panic!(r#"Driver may only return "N" or "Y""#),
                           }
                       })
                }
                SQL_ERROR => Err(Error::SqlError(self.get_diag_rec(1).unwrap())),
                _ => unreachable!(),
            }
        }
    }

    pub fn disconnect(&mut self) -> Result<()> {
        match self.raii.disconnect() {
            Return::Success(()) | Return::SuccessWithInfo(()) => Ok(()),
            Return::Error => Err(Error::SqlError(self.get_diag_rec(1).unwrap())),
        }
    }
}

impl Raii<ffi::Dbc> {
    fn disconnect(&mut self) -> Return<()> {
        match unsafe { ffi::SQLDisconnect(self.handle()) } {
            ffi::SQL_SUCCESS => Return::Success(()),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(()),
            ffi::SQL_ERROR => Return::Error,
            _ => panic!("SQLDisconnect returned unexpected result"),
        }
    }
}

