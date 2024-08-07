mod input;
mod output;
mod prepare;
mod types;
pub use self::output::Output;
pub use self::types::OdbcType;
pub use self::types::{EncodedValue, SqlDate, SqlSsTime2, SqlTime, SqlTimestamp};
use ffi::Nullable;
use ffi::SQLRETURN::*;
use std::marker::PhantomData;
use {ffi, safe, Connection, Handle, Raii, Result, Return};

// Allocate CHUNK_LEN elements at a time
const CHUNK_LEN: usize = 64;
struct Chunks<T>(Vec<Box<[T; CHUNK_LEN]>>);

/// Heap allocator that will keep allocated element pointers valid until the allocator is dropped or cleared
impl<T: Copy + Default> Chunks<T> {
    fn new() -> Chunks<T> {
        Chunks(Vec::new())
    }

    fn alloc(&mut self, i: usize, value: T) -> *mut T {
        let chunk_no = i / CHUNK_LEN;
        if self.0.len() <= chunk_no {
            // Resizing Vec that holds pointers to heap allocated arrays so we don't invalidate the references
            self.0
                .resize(chunk_no + 1, Box::new([T::default(); CHUNK_LEN]))
        }
        let v = self.0[chunk_no].get_mut(i % CHUNK_LEN).unwrap();
        *v = value;
        v as *mut T
    }

    fn clear(&mut self) {
        self.0.clear()
    }
}

/// `Statement` state used to represent a freshly allocated connection
pub enum Allocated {}
/// `Statement` state used to represent a statement with a result set cursor. A statement is most
/// likely to enter this state after a `SELECT` query.
pub type Executed = Allocated;
/// `Statement` state used to represent a statement compiled into an access plan. A statement will
/// enter this state after a call to `Statement::prepared`
pub enum Prepared {}
/// `Statement` state used to represent a statement with a result set cursor. A statement is most
/// likely to enter this state after a `SELECT` query.
pub enum HasResult {}
/// `Statement` state used to represent a statement with no result set. A statement is likely to
/// enter this state after executing e.g. a `CREATE TABLE` statement
pub enum NoResult {}

/// Holds a `Statement` after execution of a query.Allocated
///
/// A executed statement may be in one of two states. Either the statement has yielded a result set
/// or not. Keep in mind that some ODBC drivers just yield empty result sets on e.g. `INSERT`
/// Statements
pub enum ResultSetState<'a, 'b, S, AC: AutocommitMode> {
    Data(Statement<'a, 'b, S, HasResult, AC>),
    NoData(Statement<'a, 'b, S, NoResult, AC>),
}
use odbc_safe::AutocommitMode;
use std::ptr::null_mut;
pub use ResultSetState::*;

/// A `Statement` can be used to execute queries and retrieves results.
pub struct Statement<'a, 'b, S, R, AC: AutocommitMode> {
    raii: Raii<'a, ffi::Stmt>,
    state: PhantomData<S>,
    autocommit_mode: PhantomData<AC>,
    // Indicates wether there is an open result set or not associated with this statement.
    result: PhantomData<R>,
    parameters: PhantomData<&'b [u8]>,
    param_ind_buffers: Chunks<ffi::SQLLEN>,
    // encoded values are saved to use its pointer.
    encoded_values: Vec<EncodedValue>,
}

/// Used to retrieve data from the fields of a query result
pub struct Cursor<'s, 'a: 's, 'b: 's, S: 's, AC: AutocommitMode> {
    stmt: &'s mut Statement<'a, 'b, S, HasResult, AC>,
    buffer: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ColumnDescriptor {
    pub name: String,
    pub data_type: ffi::SqlDataType,
    pub column_size: Option<ffi::SQLULEN>,
    pub decimal_digits: Option<u16>,
    pub nullable: Option<bool>,
}

impl<'a, 'b, S, R, AC: AutocommitMode> Handle for Statement<'a, 'b, S, R, AC> {
    type To = ffi::Stmt;
    unsafe fn handle(&self) -> ffi::SQLHSTMT {
        self.raii.handle()
    }
}

impl<'a, 'b, S, R, AC: AutocommitMode> Statement<'a, 'b, S, R, AC> {
    fn with_raii(raii: Raii<'a, ffi::Stmt>) -> Self {
        Statement {
            raii: raii,
            autocommit_mode: PhantomData,
            state: PhantomData,
            result: PhantomData,
            parameters: PhantomData,
            param_ind_buffers: Chunks::new(),
            encoded_values: Vec::new(),
        }
    }
}

impl<'a, 'b, 'env, AC: AutocommitMode> Statement<'a, 'b, Allocated, NoResult, AC> {
    pub fn with_parent(ds: &'a Connection<'env, AC>) -> Result<Self> {
        let raii = Raii::with_parent(ds).into_result(ds)?;
        Ok(Self::with_raii(raii))
    }

    pub fn affected_row_count(&self) -> Result<ffi::SQLLEN> {
        self.raii.affected_row_count().into_result(self)
    }

    pub fn tables(
        self,
        catalog_name: &String,
        schema_name: &String,
        table_name: &String,
        table_type: &String,
    ) -> Result<Statement<'a, 'b, Executed, HasResult, AC>> {
        self.tables_str(
            catalog_name.as_str(),
            schema_name.as_str(),
            table_name.as_str(),
            table_type.as_str(),
        )
    }

    pub fn tables_str(
        self,
        catalog_name: &str,
        schema_name: &str,
        table_name: &str,
        table_type: &str,
    ) -> Result<Statement<'a, 'b, Executed, HasResult, AC>> {
        self.tables_opt_str(
            Option::Some(catalog_name),
            Option::Some(schema_name),
            Option::Some(table_name),
            table_type,
        )
    }

    pub fn tables_opt_str(
        mut self,
        catalog_name: Option<&str>,
        schema_name: Option<&str>,
        table_name: Option<&str>,
        table_type: &str,
    ) -> Result<Statement<'a, 'b, Executed, HasResult, AC>> {
        self.raii
            .tables(catalog_name, schema_name, table_name, table_type)
            .into_result(&self)?;
        Ok(Statement::with_raii(self.raii))
    }

    /// Executes a preparable statement, using the current values of the parameter marker variables
    /// if any parameters exist in the statement.
    ///
    /// `SQLExecDirect` is the fastest way to submit an SQL statement for one-time execution.
    pub fn exec_direct(
        mut self,
        statement_text: &str,
    ) -> Result<ResultSetState<'a, 'b, Executed, AC>> {
        if self.raii.exec_direct(statement_text).into_result(&self)? {
            let num_cols = self.raii.num_result_cols().into_result(&self)?;
            if num_cols > 0 {
                Ok(ResultSetState::Data(Statement::with_raii(self.raii)))
            } else {
                Ok(ResultSetState::NoData(Statement::with_raii(self.raii)))
            }
        } else {
            Ok(ResultSetState::NoData(Statement::with_raii(self.raii)))
        }
    }

    /// Executes a preparable statement, using the current values of the parameter marker variables
    /// if any parameters exist in the statement.
    ///
    /// `SQLExecDirect` is the fastest way to submit an SQL statement for one-time execution.
    pub fn exec_direct_bytes(
        mut self,
        bytes: &[u8],
    ) -> Result<ResultSetState<'a, 'b, Executed, AC>> {
        if self.raii.exec_direct_bytes(bytes).into_result(&self)? {
            let num_cols = self.raii.num_result_cols().into_result(&self)?;
            if num_cols > 0 {
                Ok(ResultSetState::Data(Statement::with_raii(self.raii)))
            } else {
                Ok(ResultSetState::NoData(Statement::with_raii(self.raii)))
            }
        } else {
            Ok(ResultSetState::NoData(Statement::with_raii(self.raii)))
        }
    }
}

impl<'a, 'b, S, AC: AutocommitMode> Statement<'a, 'b, S, HasResult, AC> {
    pub fn affected_row_count(&self) -> Result<ffi::SQLLEN> {
        self.raii.affected_row_count().into_result(self)
    }

    /// The number of columns in a result set
    ///
    /// Can be called successfully only when the statement is in the prepared, executed, or
    /// positioned state. If the statement does not return columns the result will be 0.
    pub fn num_result_cols(&self) -> Result<i16> {
        self.raii.num_result_cols().into_result(self)
    }

    /// Returns description struct for result set column with a given index. Note: indexing is starting from 1.
    pub fn describe_col(&self, idx: u16) -> Result<ColumnDescriptor> {
        self.raii.describe_col(idx).into_result(self)
    }

    /// Fetches the next rowset of data from the result set and returns data for all bound columns.
    pub fn fetch<'s>(&'s mut self) -> Result<Option<Cursor<'s, 'a, 'b, S, AC>>> {
        if self.raii.fetch().into_result(self)? {
            Ok(Some(Cursor {
                stmt: self,
                buffer: vec![0; 512],
            }))
        } else {
            Ok(None)
        }
    }

    /// Call this method to reuse the statement to execute another query.
    ///
    /// For many drivers allocating new statements is expensive. So reusing a `Statement` is usually
    /// more efficient than freeing an existing and allocating a new one. However to reuse a
    /// statement any open result sets must be closed.
    /// Only call this method if you have already read the result set returned by the previous
    /// query, or if you do no not intend to read it.
    ///
    /// # Example
    ///
    /// ```
    /// # use odbc::*;
    /// # fn reuse () -> Result<()> {
    /// let env = create_environment_v3().map_err(|e| e.unwrap())?;
    /// let conn = env.connect("TestDataSource", "", "")?;
    /// let stmt = Statement::with_parent(&conn)?;
    /// let stmt = match stmt.exec_direct("CREATE TABLE STAGE (A TEXT, B TEXT);")?{
    ///     // Some drivers will return an empty result set. We need to close it before we can use
    ///     // statement again.
    ///     Data(stmt) => stmt.close_cursor()?,
    ///     NoData(stmt) => stmt,
    /// };
    /// let stmt = stmt.exec_direct("INSERT INTO STAGE (A, B) VALUES ('Hello', 'World');")?;
    /// //...
    /// # Ok(())
    /// # };
    /// ```
    pub fn close_cursor(mut self) -> Result<Statement<'a, 'b, S, NoResult, AC>> {
        self.raii.close_cursor().into_result(&self)?;
        Ok(Statement::with_raii(self.raii))
    }
}

impl<'a, 'b, 'c, S, AC: AutocommitMode> Cursor<'a, 'b, 'c, S, AC> {
    /// Retrieves data for a single column in the result set
    ///
    /// ## Panics
    ///
    /// If you try to convert to `&str` but the data can't be converted
    /// without allocating an intermediate buffer.
    pub fn get_data<'d, T>(&'d mut self, col_or_param_num: u16) -> Result<Option<T>>
    where
        T: Output<'d>,
    {
        T::get_data(&mut self.stmt.raii, col_or_param_num, &mut self.buffer).into_result(self.stmt)
    }
}

impl<'p> Raii<'p, ffi::Stmt> {
    fn affected_row_count(&self) -> Return<ffi::SQLLEN> {
        let mut count: ffi::SQLLEN = 0;
        unsafe {
            match ffi::SQLRowCount(self.handle(), &mut count as *mut ffi::SQLLEN) {
                SQL_SUCCESS => Return::Success(count),
                SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(count),
                SQL_ERROR => Return::Error,
                r => panic!("SQLRowCount returned unexpected result: {:?}", r),
            }
        }
    }

    fn num_result_cols(&self) -> Return<i16> {
        let mut num_cols: ffi::SQLSMALLINT = 0;
        unsafe {
            match ffi::SQLNumResultCols(self.handle(), &mut num_cols as *mut ffi::SQLSMALLINT) {
                SQL_SUCCESS => Return::Success(num_cols),
                SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(num_cols),
                SQL_ERROR => Return::Error,
                r => panic!("SQLNumResultCols returned unexpected result: {:?}", r),
            }
        }
    }

    fn describe_col(&self, idx: u16) -> Return<ColumnDescriptor> {
        let mut name_buffer: [u8; 512] = [0; 512];
        let mut name_length: ffi::SQLSMALLINT = 0;
        let mut data_type: ffi::SqlDataType = ffi::SqlDataType::SQL_UNKNOWN_TYPE;
        let mut column_size: ffi::SQLULEN = 0;
        let mut decimal_digits: ffi::SQLSMALLINT = 0;
        let mut nullable: Nullable = Nullable::SQL_NULLABLE_UNKNOWN;
        unsafe {
            match ffi::SQLDescribeCol(
                self.handle(),
                idx,
                name_buffer.as_mut_ptr(),
                name_buffer.len() as ffi::SQLSMALLINT,
                &mut name_length as *mut ffi::SQLSMALLINT,
                &mut data_type as *mut ffi::SqlDataType,
                &mut column_size as *mut ffi::SQLULEN,
                &mut decimal_digits as *mut ffi::SQLSMALLINT,
                &mut nullable as *mut ffi::Nullable,
            ) {
                SQL_SUCCESS => Return::Success(ColumnDescriptor {
                    name: ::environment::DB_ENCODING
                        .decode(&name_buffer[..(name_length as usize)])
                        .0
                        .to_string(),
                    data_type: data_type,
                    column_size: if column_size == 0 {
                        None
                    } else {
                        Some(column_size)
                    },
                    decimal_digits: if decimal_digits == 0 {
                        None
                    } else {
                        Some(decimal_digits as u16)
                    },
                    nullable: match nullable {
                        Nullable::SQL_NULLABLE_UNKNOWN => None,
                        Nullable::SQL_NULLABLE => Some(true),
                        Nullable::SQL_NO_NULLS => Some(false),
                    },
                }),
                SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(ColumnDescriptor {
                    name: ::environment::DB_ENCODING
                        .decode(&name_buffer[..(name_length as usize)])
                        .0
                        .to_string(),
                    data_type: data_type,
                    column_size: if column_size == 0 {
                        None
                    } else {
                        Some(column_size)
                    },
                    decimal_digits: if decimal_digits == 0 {
                        None
                    } else {
                        Some(decimal_digits as u16)
                    },
                    nullable: match nullable {
                        Nullable::SQL_NULLABLE_UNKNOWN => None,
                        Nullable::SQL_NULLABLE => Some(true),
                        Nullable::SQL_NO_NULLS => Some(false),
                    },
                }),
                SQL_ERROR => Return::Error,
                r => panic!("SQLDescribeCol returned unexpected result: {:?}", r),
            }
        }
    }

    fn exec_direct(&mut self, statement_text: &str) -> Return<bool> {
        let bytes = unsafe { crate::environment::DB_ENCODING }
            .encode(statement_text)
            .0;

        let length = bytes.len();
        if length > ffi::SQLINTEGER::max_value() as usize {
            panic!("Statement text too long");
        }
        match unsafe {
            ffi::SQLExecDirect(self.handle(), bytes.as_ptr(), length as ffi::SQLINTEGER)
        } {
            ffi::SQL_SUCCESS => Return::Success(true),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(true),
            ffi::SQL_ERROR => Return::Error,
            ffi::SQL_NEED_DATA => panic!("SQLExecDirec returned SQL_NEED_DATA"),
            ffi::SQL_NO_DATA => Return::Success(false),
            r => panic!("SQLExecDirect returned unexpected result: {:?}", r),
        }
    }

    fn exec_direct_bytes(&mut self, bytes: &[u8]) -> Return<bool> {
        let length = bytes.len();
        if length > ffi::SQLINTEGER::max_value() as usize {
            panic!("Statement text too long");
        }
        match unsafe {
            ffi::SQLExecDirect(self.handle(), bytes.as_ptr(), length as ffi::SQLINTEGER)
        } {
            ffi::SQL_SUCCESS => Return::Success(true),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(true),
            ffi::SQL_ERROR => Return::Error,
            ffi::SQL_NEED_DATA => panic!("SQLExecDirec returned SQL_NEED_DATA"),
            ffi::SQL_NO_DATA => Return::Success(false),
            r => panic!("SQLExecDirect returned unexpected result: {:?}", r),
        }
    }

    /// Fetches the next rowset of data from the result set and returns data for all bound columns.
    fn fetch(&mut self) -> Return<bool> {
        match unsafe { ffi::SQLFetch(self.handle()) } {
            ffi::SQL_SUCCESS => Return::Success(true),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(true),
            ffi::SQL_ERROR => Return::Error,
            ffi::SQL_NO_DATA => Return::Success(false),
            r => panic!("SQLFetch returned unexpected result: {:?}", r),
        }
    }

    fn tables(
        &mut self,
        catalog_name: Option<&str>,
        schema_name: Option<&str>,
        table_name: Option<&str>,
        table_type: &str,
    ) -> Return<()> {
        unsafe {
            let mut catalog: *const odbc_sys::SQLCHAR = null_mut();
            let mut schema: *const odbc_sys::SQLCHAR = null_mut();
            let mut table: *const odbc_sys::SQLCHAR = null_mut();

            let mut catalog_size: odbc_sys::SQLSMALLINT = 0;
            let mut schema_size: odbc_sys::SQLSMALLINT = 0;
            let mut table_size: odbc_sys::SQLSMALLINT = 0;

            if catalog_name.is_some() {
                catalog = catalog_name.unwrap().as_ptr();
                catalog_size = catalog_name.unwrap().len() as odbc_sys::SQLSMALLINT;
            }

            if schema_name.is_some() {
                schema = schema_name.unwrap().as_ptr();
                schema_size = schema_name.unwrap().len() as odbc_sys::SQLSMALLINT;
            }

            if table_name.is_some() {
                table = table_name.unwrap().as_ptr();
                table_size = table_name.unwrap().len() as odbc_sys::SQLSMALLINT;
            }

            match odbc_sys::SQLTables(
                self.handle(),
                catalog,
                catalog_size,
                schema,
                schema_size,
                table,
                table_size,
                table_type.as_ptr(),
                table_type.as_bytes().len() as odbc_sys::SQLSMALLINT,
            ) {
                SQL_SUCCESS => Return::Success(()),
                SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(()),
                SQL_ERROR => Return::Error,
                r => panic!("SQLTables returned: {:?}", r),
            }
        }
    }

    fn close_cursor(&mut self) -> Return<()> {
        unsafe {
            match ffi::SQLCloseCursor(self.handle()) {
                ffi::SQL_SUCCESS => Return::Success(()),
                ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(()),
                ffi::SQL_ERROR => Return::Error,
                r => panic!("unexpected return value from SQLCloseCursor: {:?}", r),
            }
        }
    }
}

unsafe impl<'con, 'param, C, P, AC: AutocommitMode> safe::Handle
    for Statement<'con, 'param, C, P, AC>
{
    const HANDLE_TYPE: ffi::HandleType = ffi::SQL_HANDLE_STMT;

    fn handle(&self) -> ffi::SQLHANDLE {
        <Raii<ffi::Stmt> as safe::Handle>::handle(&self.raii)
    }
}
