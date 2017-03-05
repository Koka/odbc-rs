//! Reexport odbc-ffi as ffi
extern crate odbc_ffi;
pub use self::odbc_ffi::*;

// Codes used for FetchOrientation in SQLFetchScroll(), and in SQLDataSources()
pub const SQL_FETCH_NEXT: SQLUSMALLINT = 1;
pub const SQL_FETCH_FIRST: SQLUSMALLINT = 2;

// additional SQLDataSources fetch directions (ext)
pub const SQL_FETCH_FIRST_USER: SQLUSMALLINT = 31;
pub const SQL_FETCH_FIRST_SYSTEM: SQLUSMALLINT = 32;

/// Information requested by SQLGetInfo
#[repr(u16)]
#[allow(non_camel_case_types)]
pub enum InfoType {
    SQL_MAX_DRIVER_CONNECTIONS = 0,
    SQL_MAX_CONCURRENT_ACTIVITIES = 1,
    SQL_DATA_SOURCE_NAME = 2,
    SQL_FETCH_DIRECTION = 8,
    SQL_SERVER_NAME = 13,
    SQL_SEARCH_PATTERN_ESCAPE = 14,
    SQL_DBMS_NAME = 17,
    SQL_DBMS_VER = 18,
    SQL_ACCESSIBLE_TABLES = 19,
    SQL_ACCESSIBLE_PROCEDURES = 20,
    SQL_CURSOR_COMMIT_BEHAVIOR = 23,
    SQL_DATA_SOURCE_READ_ONLY = 25,
    SQL_DEFAULT_TXN_ISOLATION = 26,
    SQL_IDENTIFIER_CASE = 28,
    SQL_IDENTIFIER_QUOTE_CHAR = 29,
    SQL_MAX_COLUMN_NAME_LEN = 30,
    SQL_MAX_CURSOR_NAME_LEN = 31,
    SQL_MAX_SCHEMA_NAME_LEN = 32,
    SQL_MAX_CATALOG_NAME_LEN = 34,
    SQL_MAX_TABLE_NAME_LEN = 35,
    SQL_SCROLL_CONCURRENCY = 43,
    SQL_TRANSACTION_CAPABLE = 46,
    SQL_USER_NAME = 47,
    SQL_TRANSACTION_ISOLATION_OPTION = 72,
    SQL_INTEGRITY = 73,
    SQL_GETDATA_EXTENSIONS = 81,
    SQL_NULL_COLLATION = 85,
    SQL_ALTER_TABLE = 86,
    SQL_ORDER_BY_COLUMNS_IN_SELECT = 90,
    SQL_SPECIAL_CHARACTERS = 94,
    SQL_MAX_COLUMNS_IN_GROUP_BY = 97,
    SQL_MAX_COLUMNS_IN_INDEX = 98,
    SQL_MAX_COLUMNS_IN_ORDER_BY = 99,
    SQL_MAX_COLUMNS_IN_SELECT = 100,
    SQL_MAX_COLUMNS_IN_TABLE = 101,
    SQL_MAX_INDEX_SIZE = 102,
    SQL_MAX_ROW_SIZE = 104,
    SQL_MAX_STATEMENT_LEN = 105,
    SQL_MAX_TABLES_IN_SELECT = 106,
    SQL_MAX_USER_NAME_LEN = 107,
    SQL_OUTER_JOIN_CAPABILITIES = 115,
}
pub use self::InfoType::*;

#[cfg_attr(windows, link(name="odbc32"))]
#[cfg_attr(not(windows), link(name="odbc"))]
extern "C" {
    pub fn SQLConnect(connection_handle: SQLHDBC,
                      server_name: *const SQLCHAR,
                      name_length_1: SQLSMALLINT,
                      user_name: *const SQLCHAR,
                      name_length_2: SQLSMALLINT,
                      authentication: *const SQLCHAR,
                      name_length_3: SQLSMALLINT)
                      -> SQLRETURN;

    pub fn SQLGetInfo(connection_handle: SQLHDBC,
                      info_type: InfoType,
                      info_value: SQLPOINTER,
                      buffer_length: SQLSMALLINT,
                      string_length: *mut SQLSMALLINT)
                      -> SQLRETURN;

    pub fn SQLDataSources(EnvironmentHandle: SQLHENV,
                          Direction: SQLUSMALLINT,
                          ServerName: *mut SQLCHAR,
                          BufferLength1: SQLSMALLINT,
                          NameLength1: *mut SQLSMALLINT,
                          Description: *mut SQLCHAR,
                          BufferLength2: SQLSMALLINT,
                          NameLength2: *mut SQLSMALLINT)
                          -> SQLRETURN;

    pub fn SQLDrivers(henv: SQLHENV,
                      fDirection: SQLUSMALLINT,
                      szDriverDesc: *mut SQLCHAR,
                      cbDriverDescMax: SQLSMALLINT,
                      pcbDriverDesc: *mut SQLSMALLINT,
                      szDriverAttributes: *mut SQLCHAR,
                      cbDrvrAttrMax: SQLSMALLINT,
                      pcbDrvrAttr: *mut SQLSMALLINT)
                      -> SQLRETURN;

    pub fn SQLTables(StatementHandle: SQLHSTMT,
                     CatalogName: *const SQLCHAR,
                     NameLength1: SQLSMALLINT,
                     SchemaName: *const SQLCHAR,
                     NameLength2: SQLSMALLINT,
                     TableName: *const SQLCHAR,
                     NameLength3: SQLSMALLINT,
                     TableType: *const SQLCHAR,
                     NameLength4: SQLSMALLINT)
                     -> SQLRETURN;
}

