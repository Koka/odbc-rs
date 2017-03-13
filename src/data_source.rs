//! Holds implementation of odbc connection
use super::{ffi, Environment, Return, Result, Raii, Handle, Version3};
use std;
use std::marker::PhantomData;
use std::ptr::null_mut;

/// DataSource state, used to represent a connection to a data source.
pub enum Connected{}
/// DataSource state, used to represent a data source handle which not connected to a data source.
pub enum Disconnected{}

/// Represents a connection to an ODBC data source
pub struct DataSource<'a, S> {
    raii: Raii<ffi::Dbc>,
    // we use phantom data to tell the borrow checker that we need to keep the environment alive for
    // the lifetime of the connection
    parent: PhantomData<&'a Environment<Version3>>,
    state: PhantomData<S>,
}

impl<'a, S> Handle for DataSource<'a, S> {
    type To = ffi::Dbc;
    unsafe fn handle(&self) -> ffi::SQLHDBC {
        self.raii.handle()
    }
}

impl<'a> DataSource<'a, Disconnected> {
    /// Allocate an ODBC data source
    ///
    /// # Arguments
    /// * `env` - Environment used to allocate the data source handle.
    pub fn with_parent(env: &'a Environment<Version3>) -> Result<DataSource<'a, Disconnected>> {
        let raii = Raii::with_parent(env).into_result(env)?;
        let data_source = DataSource {
            raii: raii,
            parent: PhantomData,
            state: PhantomData
        };

        Ok(data_source)
    }

    /// Connects to an ODBC data source
    ///
    /// # Arguments
    /// * `dsn` - Data source name configured in the `odbc.ini` file
    /// * `usr` - User identifier
    /// * `pwd` - Authentication (usually password)
    pub fn connect(mut self, dsn: &str, usr: &str, pwd: &str) -> Result<DataSource<'a, Connected>> {
        self.raii.connect(dsn, usr, pwd).into_result(&self)?;
        Ok(DataSource{ raii : self.raii, parent: PhantomData, state: PhantomData})
    }

    pub fn connect_with_connection_string(mut self, connection_str : &str) -> Result<DataSource<'a, Connected>> {
        self.raii.driver_connect(connection_str).into_result(&self)?;
        Ok(DataSource{ raii : self.raii, parent: PhantomData, state: PhantomData})
    }
}

impl<'a> DataSource<'a, Connected> {
    /// `true` if the data source is set to READ ONLY mode, `false` otherwise.
    ///
    /// This characteristic pertains only to the data source itself; it is not characteristic of
    /// the driver that enables access to the data source. A driver that is read/write can be used
    /// with a data source that is read-only. If a driver is read-only, all of its data sources
    /// must be read-only.
    pub fn read_only(&self) -> Result<bool> {
        self.raii.get_info_yn(ffi::SQL_DATA_SOURCE_READ_ONLY).into_result(self)
    }

    pub fn disconnect(mut self) -> Result<DataSource<'a, Disconnected>> {
        self.raii.disconnect().into_result(&self)?;
        Ok(DataSource{ raii : self.raii, parent: PhantomData, state: PhantomData})
    }
}

impl Raii<ffi::Dbc> {
    fn get_info_yn(&self, info_type: ffi::InfoType) -> Return<bool> {
        let mut buffer: [u8; 2] = [0; 2];
        unsafe {
            match ffi::SQLGetInfo(self.handle(),
                                  info_type,
                                  buffer.as_mut_ptr() as *mut std::os::raw::c_void,
                                  buffer.len() as ffi::SQLSMALLINT,
                                  null_mut()) {
                ffi::SQL_SUCCESS => {
                    Return::Success({
                        assert!(buffer[1] == 0);
                        match buffer[0] as char {
                            'N' => false,
                            'Y' => true,
                            _ => panic!(r#"Driver may only return "N" or "Y""#),
                        }
                    })
                }
                ffi::SQL_SUCCESS_WITH_INFO => {
                    Return::SuccessWithInfo({
                        assert!(buffer[1] == 0);
                        match buffer[0] as char {
                            'N' => false,
                            'Y' => true,
                            _ => panic!(r#"Driver may only return "N" or "Y""#),
                        }
                    })
                }
                ffi::SQL_ERROR => Return::Error,
                r => panic!("SQLGetInfo returned unexpected result {:?}", r),
            }
        }
    }

    fn connect(&mut self, dsn: &str, usr: &str, pwd: &str) -> Return<()> {
        unsafe {
            match ffi::SQLConnect(self.handle(),
                                  dsn.as_ptr(),
                                  dsn.as_bytes().len() as ffi::SQLSMALLINT,
                                  usr.as_ptr(),
                                  usr.as_bytes().len() as ffi::SQLSMALLINT,
                                  pwd.as_ptr(),
                                  pwd.as_bytes().len() as ffi::SQLSMALLINT) {
                ffi::SQL_SUCCESS => Return::Success(()),
                ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(()),
                _ => Return::Error,
            }
        }
    }

    fn driver_connect(&mut self, connection_str: &str) -> Return<()> {
        let length = connection_str.as_bytes().len();
        if length > ffi::SQLSMALLINT::max_value() as usize {
            panic!("Connection string is too long");
        }
        match unsafe {
            ffi::SQLDriverConnect(self.handle(),
                                  null_mut(),
                                  connection_str.as_ptr(),
                                  length as ffi::SQLSMALLINT,
                                  null_mut(),
                                  0,
                                  null_mut(),
                                  ffi::SQL_DRIVER_NOPROMPT)
        }{
            ffi::SQL_SUCCESS => Return::Success(()),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(()),
            ffi::SQL_ERROR => Return::Error,
            r => panic!("SQLDriverConnect returned unexpected {:?}", r),
        }
    }

    fn disconnect(&mut self) -> Return<()> {
        match unsafe { ffi::SQLDisconnect(self.handle()) } {
            ffi::SQL_SUCCESS => Return::Success(()),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(()),
            ffi::SQL_ERROR => Return::Error,
            _ => panic!("SQLDisconnect returned unexpected result"),
        }
    }
}
