//! Holds implementation of odbc connection
use super::{ffi, safe, Environment, Handle, Result, Version3};
use super::result::{into_result, into_result_with};

/// DataSource state used to represent a connection to a data source.
pub use odbc_safe::Connected;
/// DataSource state used to represent a data source handle which not connected to a data source.
pub use odbc_safe::Unconnected;

/// Represents a connection to an ODBC data source
///
/// A `DataSource` is in one of two states `Connected` or `Unconnected`. These are known at
/// compile time. Every new `DataSource` starts out as `Unconnected`. To do execute a query it
/// needs to be connected. You can achieve this by calling e.g. `connect` and capture the result in
/// a new binding which will be of type `DataSource::<Connected>`.
pub struct DataSource<'env> {
    safe: safe::Connection<'env>,
}

impl<'env> Handle for DataSource<'env> {
    type To = ffi::Dbc;
    unsafe fn handle(&self) -> ffi::SQLHDBC {
        self.safe.as_raw()
    }
}

impl Environment<Version3> {
    /// Connects to an ODBC data source
    ///
    /// # Arguments
    /// * `dsn` - Data source name configured in the `odbc.ini` file
    /// * `usr` - User identifier
    /// * `pwd` - Authentication (usually password)
    pub fn connect<'env>(&'env self, dsn: &str, usr: &str, pwd: &str) -> Result<DataSource<'env>> {
        let safe = into_result_with(self, safe::DataSource::with_parent(self.as_safe()))?;
        let safe = into_result(safe.connect(dsn, usr, pwd))?;
        Ok(DataSource { safe })
    }

    pub fn connect_with_connection_string<'env>(
        &'env self,
        connection_str: &str,
    ) -> Result<DataSource<'env>> {
        let safe = into_result_with(self, safe::DataSource::with_parent(self.as_safe()))?;
        let safe = into_result(safe.connect_with_connection_string(connection_str))?;
        Ok(DataSource { safe })
    }
}

impl<'env> DataSource<'env> {
    /// `true` if the data source is set to READ ONLY mode, `false` otherwise.
    ///
    /// This characteristic pertains only to the data source itself; it is not characteristic of
    /// the driver that enables access to the data source. A driver that is read/write can be used
    /// with a data source that is read-only. If a driver is read-only, all of its data sources
    /// must be read-only.
    pub fn is_read_only(&mut self) -> Result<bool> {
        let ret = self.safe.is_read_only();
        into_result_with(&self.safe, ret)
    }

    /// Closes the connection to the DataSource. If not called explicitly this the disconnect will
    /// be invoked by `drop()`
    pub fn disconnect(self) -> Result<()> {
        into_result(self.safe.disconnect())?;
        Ok(())
    }
}

unsafe impl<'env> safe::Handle for DataSource<'env> {
    fn handle(&self) -> ffi::SQLHANDLE {
        self.safe.as_raw() as ffi::SQLHANDLE
    }

    fn handle_type() -> ffi::HandleType {
        ffi::SQL_HANDLE_DBC
    }
}
