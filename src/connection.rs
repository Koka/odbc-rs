//! Holds implementation of odbc connection
use super::result::{into_result, into_result_with};
use super::{ffi, safe, Environment, Handle, Result, Version3};
use odbc_safe::{AutocommitMode, AutocommitOff, AutocommitOn};

/// Represents a connection to an ODBC data source
#[derive(Debug)]
pub struct Connection<'env, AC: AutocommitMode> {
    safe: safe::Connection<'env, AC>,
}

impl<'env, AC: AutocommitMode> Handle for Connection<'env, AC> {
    type To = ffi::Dbc;
    unsafe fn handle(&self) -> ffi::SQLHDBC {
        self.safe.as_raw()
    }
}

// Place Constructors into environment, to make them easier to discover
impl Environment<Version3> {
    /// Connects to an ODBC data source
    ///
    /// # Arguments
    /// * `dsn` - Data source name configured in the `odbc.ini` file
    /// * `usr` - User identifier
    /// * `pwd` - Authentication (usually password)
    pub fn connect<'env>(
        &'env self,
        dsn: &str,
        usr: &str,
        pwd: &str,
    ) -> Result<Connection<'env, AutocommitOn>> {
        let safe = into_result_with(self, safe::DataSource::with_parent(self.as_safe()))?;
        let safe = into_result(safe.connect(dsn, usr, pwd))?;
        Ok(Connection { safe })
    }

    /// Connects to an ODBC data source using a connection string
    ///
    /// See [SQLDriverConnect][1] for the syntax.
    /// [1]: https://docs.microsoft.com/en-us/sql/odbc/reference/syntax/sqldriverconnect-function
    pub fn connect_with_connection_string<'env>(
        &'env self,
        connection_str: &str,
    ) -> Result<Connection<'env, AutocommitOn>> {
        let safe = into_result_with(self, safe::DataSource::with_parent(self.as_safe()))?;
        let safe = into_result(safe.connect_with_connection_string(connection_str))?;
        Ok(Connection { safe })
    }
}

impl<'env> Connection<'env, AutocommitOn> {
    pub fn disable_autocommit(self) -> std::result::Result<Connection<'env, AutocommitOff>, Self> {
        let ret = self.safe.disable_autocommit();
        match ret {
            safe::Return::Success(value) => Ok(Connection { safe: value }),
            safe::Return::Info(value) => Ok(Connection { safe: value }),
            safe::Return::Error(value) => Err(Connection { safe: value }),
        }
    }
}

impl<'env> Connection<'env, AutocommitOff> {
    pub fn enable_autocommit(self) -> std::result::Result<Connection<'env, AutocommitOn>, Self> {
        let ret = self.safe.enable_autocommit();
        match ret {
            safe::Return::Success(value) => Ok(Connection { safe: value }),
            safe::Return::Info(value) => Ok(Connection { safe: value }),
            safe::Return::Error(value) => Err(Connection { safe: value }),
        }
    }

    pub fn commit(&mut self) -> Result<()> {
        let ret = self.safe.commit();
        into_result_with(&self.safe, ret)
    }

    pub fn rollback(&mut self) -> Result<()> {
        let ret = self.safe.rollback();
        into_result_with(&self.safe, ret)
    }
}

impl<'env, AC: AutocommitMode> Connection<'env, AC> {
    /// `true` if the data source is set to READ ONLY mode, `false` otherwise.
    ///
    /// This characteristic pertains only to the data source itself; it is not characteristic of
    /// the driver that enables access to the data source. A driver that is read/write can be used
    /// with a data source that is read-only. If a driver is read-only, all of its data sources
    /// must be read-only.
    pub fn is_read_only(&mut self) -> Result<bool> {
        // The mutability on is_read_only is really an eyesore. Not only to clippy. But we would
        // have to introduce a cell around `self.safe`, and be careful not to change essential
        // state in the error path. For now the trouble does not seem worth it.
        let ret = self.safe.is_read_only();
        into_result_with(&self.safe, ret)
    }

    /// Closes the connection to the data source. If not called explicitly the disconnect will be
    /// invoked implicitly by `drop()`
    pub fn disconnect(self) -> Result<()> {
        into_result(self.safe.disconnect())?;
        Ok(())
    }
}

unsafe impl<'env, AC: AutocommitMode> safe::Handle for Connection<'env, AC> {
    const HANDLE_TYPE: ffi::HandleType = ffi::SQL_HANDLE_DBC;

    fn handle(&self) -> ffi::SQLHANDLE {
        self.safe.as_raw() as ffi::SQLHANDLE
    }
}
