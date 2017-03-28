var searchIndex = {};
searchIndex["log"] = {"doc":"A lightweight logging facade.","items":[[3,"LogRecord","log","The \"payload\" of a log message.",null,null],[3,"LogMetadata","","Metadata about a log message.",null,null],[3,"LogLocation","","The location of a log message.",null,null],[3,"MaxLogLevelFilter","","A token providing read and write access to the global maximum log level filter.",null,null],[3,"SetLoggerError","","The type returned by `set_logger` if `set_logger` has already been called.",null,null],[3,"ShutdownLoggerError","","The type returned by `shutdown_logger_raw` if `shutdown_logger_raw` has already been called or if `set_logger_raw` has not been called yet.",null,null],[4,"LogLevel","","An enum representing the available verbosity levels of the logging framework",null,null],[13,"Error","","The \"error\" level.",0,null],[13,"Warn","","The \"warn\" level.",0,null],[13,"Info","","The \"info\" level.",0,null],[13,"Debug","","The \"debug\" level.",0,null],[13,"Trace","","The \"trace\" level.",0,null],[4,"LogLevelFilter","","An enum representing the available verbosity level filters of the logging framework.",null,null],[13,"Off","","A level lower than all log levels.",1,null],[13,"Error","","Corresponds to the `Error` log level.",1,null],[13,"Warn","","Corresponds to the `Warn` log level.",1,null],[13,"Info","","Corresponds to the `Info` log level.",1,null],[13,"Debug","","Corresponds to the `Debug` log level.",1,null],[13,"Trace","","Corresponds to the `Trace` log level.",1,null],[5,"max_log_level","","Returns the current maximum log level.",null,{"inputs":[],"output":{"name":"loglevelfilter"}}],[5,"set_logger","","Sets the global logger.",null,{"inputs":[{"name":"m"}],"output":{"name":"result"}}],[5,"set_logger_raw","","Sets the global logger from a raw pointer.",null,{"inputs":[{"name":"m"}],"output":{"name":"result"}}],[5,"shutdown_logger","","Shuts down the global logger.",null,{"inputs":[],"output":{"name":"result"}}],[5,"shutdown_logger_raw","","Shuts down the global logger.",null,{"inputs":[],"output":{"name":"result"}}],[8,"Log","","A trait encapsulating the operations required of a logger",null,null],[10,"enabled","","Determines if a log message with the specified metadata would be logged.",2,{"inputs":[{"name":"self"},{"name":"logmetadata"}],"output":{"name":"bool"}}],[10,"log","","Logs the `LogRecord`.",2,{"inputs":[{"name":"self"},{"name":"logrecord"}],"output":null}],[11,"fmt","","",0,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"clone","","",0,{"inputs":[{"name":"self"}],"output":{"name":"loglevel"}}],[11,"eq","","",0,{"inputs":[{"name":"self"},{"name":"loglevel"}],"output":{"name":"bool"}}],[11,"eq","","",0,{"inputs":[{"name":"self"},{"name":"loglevelfilter"}],"output":{"name":"bool"}}],[11,"partial_cmp","","",0,{"inputs":[{"name":"self"},{"name":"loglevel"}],"output":{"name":"option"}}],[11,"partial_cmp","","",0,{"inputs":[{"name":"self"},{"name":"loglevelfilter"}],"output":{"name":"option"}}],[11,"cmp","","",0,{"inputs":[{"name":"self"},{"name":"loglevel"}],"output":{"name":"ordering"}}],[11,"from_str","","",0,{"inputs":[{"name":"str"}],"output":{"name":"result"}}],[11,"fmt","","",0,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"max","","Returns the most verbose logging level.",0,{"inputs":[],"output":{"name":"loglevel"}}],[11,"to_log_level_filter","","Converts the `LogLevel` to the equivalent `LogLevelFilter`.",0,{"inputs":[{"name":"self"}],"output":{"name":"loglevelfilter"}}],[11,"fmt","","",1,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"clone","","",1,{"inputs":[{"name":"self"}],"output":{"name":"loglevelfilter"}}],[11,"eq","","",1,{"inputs":[{"name":"self"},{"name":"loglevelfilter"}],"output":{"name":"bool"}}],[11,"eq","","",1,{"inputs":[{"name":"self"},{"name":"loglevel"}],"output":{"name":"bool"}}],[11,"partial_cmp","","",1,{"inputs":[{"name":"self"},{"name":"loglevelfilter"}],"output":{"name":"option"}}],[11,"partial_cmp","","",1,{"inputs":[{"name":"self"},{"name":"loglevel"}],"output":{"name":"option"}}],[11,"cmp","","",1,{"inputs":[{"name":"self"},{"name":"loglevelfilter"}],"output":{"name":"ordering"}}],[11,"from_str","","",1,{"inputs":[{"name":"str"}],"output":{"name":"result"}}],[11,"fmt","","",1,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"max","","Returns the most verbose logging level filter.",1,{"inputs":[],"output":{"name":"loglevelfilter"}}],[11,"to_log_level","","Converts `self` to the equivalent `LogLevel`.",1,{"inputs":[{"name":"self"}],"output":{"name":"option"}}],[11,"args","","The message body.",3,{"inputs":[{"name":"self"}],"output":{"name":"arguments"}}],[11,"metadata","","Metadata about the log directive.",3,{"inputs":[{"name":"self"}],"output":{"name":"logmetadata"}}],[11,"location","","The location of the log directive.",3,{"inputs":[{"name":"self"}],"output":{"name":"loglocation"}}],[11,"level","","The verbosity level of the message.",3,{"inputs":[{"name":"self"}],"output":{"name":"loglevel"}}],[11,"target","","The name of the target of the directive.",3,{"inputs":[{"name":"self"}],"output":{"name":"str"}}],[11,"level","","The verbosity level of the message.",4,{"inputs":[{"name":"self"}],"output":{"name":"loglevel"}}],[11,"target","","The name of the target of the directive.",4,{"inputs":[{"name":"self"}],"output":{"name":"str"}}],[11,"clone","","",5,{"inputs":[{"name":"self"}],"output":{"name":"loglocation"}}],[11,"fmt","","",5,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"module_path","","The module path of the message.",5,{"inputs":[{"name":"self"}],"output":{"name":"str"}}],[11,"file","","The source file containing the message.",5,{"inputs":[{"name":"self"}],"output":{"name":"str"}}],[11,"line","","The line containing the message.",5,{"inputs":[{"name":"self"}],"output":{"name":"u32"}}],[11,"fmt","","",6,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"get","","Gets the current maximum log level filter.",6,{"inputs":[{"name":"self"}],"output":{"name":"loglevelfilter"}}],[11,"set","","Sets the maximum log level.",6,{"inputs":[{"name":"self"},{"name":"loglevelfilter"}],"output":null}],[11,"fmt","","",7,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"fmt","","",7,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"description","","",7,{"inputs":[{"name":"self"}],"output":{"name":"str"}}],[11,"fmt","","",8,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"fmt","","",8,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"description","","",8,{"inputs":[{"name":"self"}],"output":{"name":"str"}}],[14,"log","","The standard logging macro.",null,null],[14,"error","","Logs a message at the error level.",null,null],[14,"warn","","Logs a message at the warn level.",null,null],[14,"info","","Logs a message at the info level.",null,null],[14,"debug","","Logs a message at the debug level.",null,null],[14,"trace","","Logs a message at the trace level.",null,null],[14,"log_enabled","","Determines if a message logged at the specified level in that module will be logged.",null,null]],"paths":[[4,"LogLevel"],[4,"LogLevelFilter"],[8,"Log"],[3,"LogRecord"],[3,"LogMetadata"],[3,"LogLocation"],[3,"MaxLogLevelFilter"],[3,"SetLoggerError"],[3,"ShutdownLoggerError"]]};
searchIndex["odbc"] = {"doc":"ODBC Open Database Connectivity or short ODBC is a low level high performance interface introduced by Microsoft to access relational data stores. This crate wraps the raw C interface and is intended to be usable in safe and idiomatic Rust.","items":[[3,"DiagnosticRecord","odbc","ODBC Diagnostic Record",null,null],[3,"EnvAllocError","","Environment allocation error",null,null],[3,"DataSourceInfo","","Holds name and description of a datasource",null,null],[12,"server_name","","Name of the data source",0,null],[12,"driver","","Description of the data source",0,null],[3,"DriverInfo","","Struct holding information available on a driver.",null,null],[12,"description","","Name of the odbc driver",1,null],[12,"attributes","","List of attributes of the odbc driver",1,null],[3,"Environment","","Handle to an ODBC Environment",null,null],[3,"DataSource","","Represents a connection to an ODBC data source",null,null],[3,"Statement","","RAII wrapper around ODBC statement",null,null],[3,"Cursor","","Used to retrieve data from the fields of a query resul",null,null],[4,"NoVersion","","Environment state used to represent that no odbc version has been set.",null,null],[4,"Version3","","Environment state used to represent that environment has been set to odbc version 3",null,null],[4,"Connected","","DataSource state used to represent a connection to a data source.",null,null],[4,"Disconnected","","DataSource state used to represent a data source handle which not connected to a data source.",null,null],[4,"Allocated","","`Statement` state used to represent a freshly allocated connection",null,null],[4,"HasResult","","`Statement` state used to represent a statement with a result set cursor",null,null],[4,"Executed","","Holds a `Statement` after execution of a query.Allocated",null,null],[13,"Data","","",2,null],[13,"NoData","","",2,null],[0,"ffi","","Reexport odbc-sys as ffi",null,null],[11,"fmt","","",3,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"fmt","","",3,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"description","","",3,{"inputs":[{"name":"self"}],"output":{"name":"str"}}],[11,"cause","","",3,{"inputs":[{"name":"self"}],"output":{"name":"option"}}],[11,"fmt","","",4,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"fmt","","",4,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"description","","",4,{"inputs":[{"name":"self"}],"output":{"name":"str"}}],[11,"cause","","",4,{"inputs":[{"name":"self"}],"output":{"name":"option"}}],[11,"clone","","",0,{"inputs":[{"name":"self"}],"output":{"name":"datasourceinfo"}}],[11,"fmt","","",0,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"eq","","",0,{"inputs":[{"name":"self"},{"name":"datasourceinfo"}],"output":{"name":"bool"}}],[11,"ne","","",0,{"inputs":[{"name":"self"},{"name":"datasourceinfo"}],"output":{"name":"bool"}}],[11,"clone","","",1,{"inputs":[{"name":"self"}],"output":{"name":"driverinfo"}}],[11,"fmt","","",1,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"eq","","",1,{"inputs":[{"name":"self"},{"name":"driverinfo"}],"output":{"name":"bool"}}],[11,"ne","","",1,{"inputs":[{"name":"self"},{"name":"driverinfo"}],"output":{"name":"bool"}}],[11,"drivers","","Stores all driver description and attributes in a Vec",5,{"inputs":[{"name":"self"}],"output":{"name":"result"}}],[11,"data_sources","","Stores all data source server names and descriptions in a Vec",5,{"inputs":[{"name":"self"}],"output":{"name":"result"}}],[11,"system_data_sources","","Stores all sytem data source server names and descriptions in a Vec",5,{"inputs":[{"name":"self"}],"output":{"name":"result"}}],[11,"user_data_sources","","Stores all user data source server names and descriptions in a Vec",5,{"inputs":[{"name":"self"}],"output":{"name":"result"}}],[11,"handle","","",5,{"inputs":[{"name":"self"}],"output":{"name":"sqlhenv"}}],[11,"new","","Allocates a new ODBC Environment",5,{"inputs":[],"output":{"name":"result"}}],[11,"set_odbc_version_3","","Tells the driver(s) that we will use features of up to ODBC version 3",5,{"inputs":[{"name":"self"}],"output":{"name":"result"}}],[11,"handle","","",6,{"inputs":[{"name":"self"}],"output":{"name":"sqlhdbc"}}],[11,"drop","","",6,{"inputs":[{"name":"self"}],"output":null}],[11,"with_parent","","Allocate an ODBC data source",6,{"inputs":[{"name":"environment"}],"output":{"name":"result"}}],[11,"connect","","Connects to an ODBC data source",6,{"inputs":[{"name":"self"},{"name":"str"},{"name":"str"},{"name":"str"}],"output":{"name":"result"}}],[11,"connect_with_connection_string","","",6,{"inputs":[{"name":"self"},{"name":"str"}],"output":{"name":"result"}}],[11,"read_only","","`true` if the data source is set to READ ONLY mode, `false` otherwise.",6,{"inputs":[{"name":"self"}],"output":{"name":"result"}}],[11,"disconnect","","Closes the connection to the DataSource. If not called explicitly this the disconnect will be invoked by `drop()`",6,{"inputs":[{"name":"self"}],"output":{"name":"result"}}],[11,"bind_parameter","","Binds a parameter to a parameter marker in an SQL statement.",7,{"inputs":[{"name":"self"},{"name":"u16"},{"name":"t"}],"output":{"name":"result"}}],[11,"handle","","",7,{"inputs":[{"name":"self"}],"output":{"name":"sqlhstmt"}}],[11,"with_parent","","",7,{"inputs":[{"name":"datasource"}],"output":{"name":"result"}}],[11,"tables","","",7,{"inputs":[{"name":"self"}],"output":{"name":"result"}}],[11,"exec_direct","","Executes a preparable statement, using the current values of the parameter marker variables if any parameters exist in the statement.",7,{"inputs":[{"name":"self"},{"name":"str"}],"output":{"name":"result"}}],[11,"num_result_cols","","The number of columns in a result set",7,{"inputs":[{"name":"self"}],"output":{"name":"result"}}],[11,"fetch","","Fetches the next rowset of data from the result set and returns data for all bound columns.",7,{"inputs":[{"name":"self"}],"output":{"name":"result"}}],[11,"close_cursor","","Call this method to reuse the statement to execute another query.",7,{"inputs":[{"name":"self"}],"output":{"name":"result"}}],[11,"get_data","","Retrieves data for a single column in the result set",8,{"inputs":[{"name":"self"},{"name":"u16"}],"output":{"name":"result"}}],[6,"Result","","Result type returned by most functions in this crate",null,null],[8,"GetDiagRec","","",null,null],[10,"get_diag_rec","","Retrieves a diagnostic record",9,{"inputs":[{"name":"self"},{"name":"i16"}],"output":{"name":"option"}}],[8,"Output","","Indicates that a type can be retrieved using `Cursor::get_data`",null,null],[10,"get_data","","",10,null],[8,"InputParameter","","Allows types to be used with `Statement::bind_parameter`",null,null],[10,"c_data_type","","",11,{"inputs":[{"name":"self"}],"output":{"name":"sqlcdatatype"}}],[10,"column_size","","",11,{"inputs":[{"name":"self"}],"output":{"name":"sqlulen"}}],[10,"decimal_digits","","",11,{"inputs":[{"name":"self"}],"output":{"name":"sqlsmallint"}}],[10,"value_ptr","","",11,{"inputs":[{"name":"self"}],"output":{"name":"sqlpointer"}}],[10,"indicator","","",11,{"inputs":[{"name":"self"}],"output":{"name":"sqllen"}}],[8,"Handle","","Reflects the ability of a type to expose a valid handle",null,null],[16,"To","","",12,null],[10,"handle","","Returns a valid handle to the odbc type.",12,null]],"paths":[[3,"DataSourceInfo"],[3,"DriverInfo"],[4,"Executed"],[3,"DiagnosticRecord"],[3,"EnvAllocError"],[3,"Environment"],[3,"DataSource"],[3,"Statement"],[3,"Cursor"],[8,"GetDiagRec"],[8,"Output"],[8,"InputParameter"],[8,"Handle"]]};
searchIndex["odbc_sys"] = {"doc":"ODBC types those representation is compatible with the ODBC C API.","items":[[4,"SQLRETURN","odbc_sys","Indicates the overall success or failure of the function",null,null],[13,"SQL_INVALID_HANDLE","","Function failed due to an invalid environment, connection, statement, or descriptor handle",0,null],[13,"SQL_ERROR","","Function failed",0,null],[13,"SQL_SUCCESS","","Function completed successfully",0,null],[13,"SQL_SUCCESS_WITH_INFO","","Function completed successfully, possibly with a nonfatal error (warning)",0,null],[13,"SQL_STILL_EXECUTING","","A function that was started asynchronously is still executing",0,null],[13,"SQL_NEED_DATA","","More data is needed",0,null],[13,"SQL_NO_DATA","","No more data was available",0,null],[13,"SQL_PARAM_DATA_AVAILABLE","","",0,null],[4,"InfoType","","Information requested by SQLGetInfo",null,null],[13,"SQL_MAX_DRIVER_CONNECTIONS","","",1,null],[13,"SQL_MAX_CONCURRENT_ACTIVITIES","","",1,null],[13,"SQL_DATA_SOURCE_NAME","","",1,null],[13,"SQL_SERVER_NAME","","",1,null],[13,"SQL_SEARCH_PATTERN_ESCAPE","","",1,null],[13,"SQL_DBMS_NAME","","",1,null],[13,"SQL_DBMS_VER","","",1,null],[13,"SQL_ACCESSIBLE_TABLES","","",1,null],[13,"SQL_ACCESSIBLE_PROCEDURES","","",1,null],[13,"SQL_CURSOR_COMMIT_BEHAVIOR","","",1,null],[13,"SQL_DATA_SOURCE_READ_ONLY","","",1,null],[13,"SQL_DEFAULT_TXN_ISOLATION","","",1,null],[13,"SQL_IDENTIFIER_CASE","","",1,null],[13,"SQL_IDENTIFIER_QUOTE_CHAR","","",1,null],[13,"SQL_MAX_COLUMN_NAME_LEN","","",1,null],[13,"SQL_MAX_CURSOR_NAME_LEN","","",1,null],[13,"SQL_MAX_SCHEMA_NAME_LEN","","",1,null],[13,"SQL_MAX_CATALOG_NAME_LEN","","",1,null],[13,"SQL_MAX_TABLE_NAME_LEN","","",1,null],[13,"SQL_TRANSACTION_CAPABLE","","",1,null],[13,"SQL_USER_NAME","","",1,null],[13,"SQL_TRANSACTION_ISOLATION_OPTION","","",1,null],[13,"SQL_INTEGRITY","","",1,null],[13,"SQL_GETDATA_EXTENSIONS","","",1,null],[13,"SQL_NULL_COLLATION","","",1,null],[13,"SQL_ALTER_TABLE","","",1,null],[13,"SQL_ORDER_BY_COLUMNS_IN_SELECT","","",1,null],[13,"SQL_SPECIAL_CHARACTERS","","",1,null],[13,"SQL_MAX_COLUMNS_IN_GROUP_BY","","",1,null],[13,"SQL_MAX_COLUMNS_IN_INDEX","","",1,null],[13,"SQL_MAX_COLUMNS_IN_ORDER_BY","","",1,null],[13,"SQL_MAX_COLUMNS_IN_SELECT","","",1,null],[13,"SQL_MAX_COLUMNS_IN_TABLE","","",1,null],[13,"SQL_MAX_INDEX_SIZE","","",1,null],[13,"SQL_MAX_ROW_SIZE","","",1,null],[13,"SQL_MAX_STATEMENT_LEN","","",1,null],[13,"SQL_MAX_TABLES_IN_SELECT","","",1,null],[13,"SQL_MAX_USER_NAME_LEN","","",1,null],[13,"SQL_OUTER_JOIN_CAPABILITIES","","",1,null],[13,"SQL_XOPEN_CLI_YEAR","","",1,null],[13,"SQL_CURSOR_SENSITIVITY","","",1,null],[13,"SQL_DESCRIBE_PARAMETER","","",1,null],[13,"SQL_CATALOG_NAME","","",1,null],[13,"SQL_COLLATION_SEQ","","",1,null],[13,"SQL_MAX_IDENTIFIER_LEN","","",1,null],[4,"FetchOrientation","","Codes used for FetchOrientation in `SQLFetchScroll`, `SQLDataSources` and in `SQLDrivers`",null,null],[13,"SQL_FETCH_NEXT","","",2,null],[13,"SQL_FETCH_FIRST","","",2,null],[13,"SQL_FETCH_LAST","","",2,null],[13,"SQL_FETCH_PRIOR","","",2,null],[13,"SQL_FETCH_ABSOLUTE","","",2,null],[13,"SQL_FETCH_RELATIVE","","",2,null],[13,"SQL_FETCH_FIRST_USER","","",2,null],[13,"SQL_FETCH_FIRST_SYSTEM","","",2,null],[4,"EnvironmentAttribute","","Governs behaviour of EnvironmentAttribute",null,null],[13,"SQL_ATTR_ODBC_VERSION","","",3,null],[13,"SQL_ATTR_CONNECTION_POOLING","","",3,null],[13,"SQL_ATTR_CP_MATCH","","",3,null],[13,"SQL_ATTR_APPLICATION_KEY","","",3,null],[13,"SQL_ATTR_OUTPUT_NTS","","",3,null],[4,"SqlCDataType","","The C data type is specified in the SQLBindCol and SQLGetData functions with the TargetType argument and in the SQLBindParameter function with the ValueType argument.",null,null],[13,"SQL_C_UTINYINT","","",4,null],[13,"SQL_C_UBIGINT","","",4,null],[13,"SQL_C_STINYINT","","",4,null],[13,"SQL_C_SBIGINT","","",4,null],[13,"SQL_C_ULONG","","",4,null],[13,"SQL_C_USHORT","","",4,null],[13,"SQL_C_SLONG","","",4,null],[13,"SQL_C_SSHORT","","",4,null],[13,"SQL_C_GUID","","",4,null],[13,"SQL_C_BIT","","",4,null],[13,"SQL_C_BINARY","","",4,null],[13,"SQL_C_CHAR","","`SQLCHAR` - CHAR, VARCHAR, DECIMAL, NUMERIC",4,null],[13,"SQL_C_NUMERIC","","",4,null],[13,"SQL_C_FLOAT","","",4,null],[13,"SQL_C_DOUBLE","","",4,null],[13,"SQL_C_DATE","","",4,null],[13,"SQL_C_TIME","","",4,null],[13,"SQL_C_TIMESTAMP","","",4,null],[13,"SQL_C_TYPE_DATE","","",4,null],[13,"SQL_C_TYPE_TIME","","",4,null],[13,"SQL_C_TYPE_TIMESTAMP","","",4,null],[13,"SQL_C_DEFAULT","","",4,null],[13,"SQL_C_INTERVAL_YEAR","","",4,null],[13,"SQL_C_INTERVAL_MONTH","","",4,null],[13,"SQL_C_INTERVAL_DAY","","",4,null],[13,"SQL_C_INTERVAL_HOUR","","",4,null],[13,"SQL_C_INTERVAL_MINUTE","","",4,null],[13,"SQL_C_INTERVAL_SECOND","","",4,null],[13,"SQL_C_INTERVAL_YEAR_TO_MONTH","","",4,null],[13,"SQL_C_INTERVAL_DAY_TO_HOUR","","",4,null],[13,"SQL_C_INTERVAL_DAY_TO_MINUTE","","",4,null],[13,"SQL_C_INTERVAL_DAY_TO_SECOND","","",4,null],[13,"SQL_C_INTERVAL_HOUR_TO_MINUTE","","",4,null],[13,"SQL_C_INTERVAL_HOUR_TO_SECOND","","",4,null],[13,"SQL_C_INTERVAL_MINUTE_TO_SECOND","","",4,null],[4,"InputOutput","","Used by `SQLBindParameter`.",null,null],[13,"SQL_PARAM_TYPE_UNKNOWN","","",5,null],[13,"SQL_PARAM_INPUT","","",5,null],[13,"SQL_PARAM_INPUT_OUTPUT","","",5,null],[13,"SQL_RESULT_COL","","",5,null],[13,"SQL_PARAM_OUTPUT","","",5,null],[13,"SQL_RETURN_VALUE","","",5,null],[13,"SQL_PARAM_INPUT_OUTPUT_STREAM","","",5,null],[13,"SQL_PARAM_OUTPUT_STREAM","","",5,null],[4,"Obj","","",null,null],[4,"Env","","",null,null],[4,"Dbc","","",null,null],[4,"Stmt","","",null,null],[4,"SqlDataType","","SQL Data Types",null,null],[13,"SQL_UNKNOWN_TYPE","","",6,null],[13,"SQL_CHAR","","",6,null],[13,"SQL_NUMERIC","","",6,null],[13,"SQL_DECIMAL","","",6,null],[13,"SQL_INTEGER","","",6,null],[13,"SQL_SMALLINT","","",6,null],[13,"SQL_FLOAT","","",6,null],[13,"SQL_REAL","","",6,null],[13,"SQL_DOUBLE","","",6,null],[13,"SQL_DATETIME","","",6,null],[13,"SQL_VARCHAR","","",6,null],[4,"HandleType","","Represented in C headers as SQLSMALLINT",null,null],[13,"SQL_HANDLE_ENV","","",7,null],[13,"SQL_HANDLE_DBC","","",7,null],[13,"SQL_HANDLE_STMT","","",7,null],[13,"SQL_HANDLE_DESC","","",7,null],[4,"SqlDriverConnectOption","","Options for `SQLDriverConnect`",null,null],[13,"SQL_DRIVER_NOPROMPT","","",8,null],[13,"SQL_DRIVER_COMPLETE","","",8,null],[13,"SQL_DRIVER_PROMPT","","",8,null],[13,"SQL_DRIVER_COMPLETE_REQUIRED","","",8,null],[5,"SQLAllocHandle","","Allocates an environment, connection, statement, or descriptor handle.",null,null],[5,"SQLFreeHandle","","Frees resources associated with a specific environment, connection, statement, or descriptor handle.",null,null],[5,"SQLSetEnvAttr","","Sets attributes that govern aspects of environments",null,null],[5,"SQLDisconnect","","Closes the connection associated with a specific connection handle.",null,null],[5,"SQLGetDiagRec","","",null,null],[5,"SQLExecDirect","","",null,null],[5,"SQLNumResultCols","","Returns the number of columns in a result set",null,null],[5,"SQLGetData","","",null,null],[5,"SQLFetch","","SQLFetch fetches the next rowset of data from the result set and returns data for all bound columns.",null,null],[5,"SQLGetInfo","","Returns general information about the driver and data source associated with a connection",null,null],[5,"SQLConnect","","SQLConnect establishes connections to a driver and a data source.",null,null],[5,"SQLTables","","Returns the list of table, catalog, or schema names, and table types, stored in a specific data source. The driver returns the information as a result set",null,null],[5,"SQLDataSources","","Returns information about a data source. This function is implemented only by the Driver Manager.",null,null],[5,"SQLDriverConnect","","An alternative to `SQLConnect`. It supports data sources that require more connection information than the three arguments in `SQLConnect`, dialog boxes to prompt the user for all connection information, and data sources that are not defined in the system information",null,null],[5,"SQLDrivers","","Lists driver descriptions and driver attribute keywords. This function is implemented only by the Driver Manager.",null,null],[5,"SQLCloseCursor","","Closes a cursor that has been opened on a statement and discards pending results.",null,null],[5,"SQLBindParameter","","Binds a buffer to a parameter marker in an SQL statement",null,null],[11,"fmt","","",0,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"eq","","",0,{"inputs":[{"name":"self"},{"name":"sqlreturn"}],"output":{"name":"bool"}}],[11,"clone","","",0,{"inputs":[{"name":"self"}],"output":{"name":"sqlreturn"}}],[11,"fmt","","",1,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"eq","","",1,{"inputs":[{"name":"self"},{"name":"infotype"}],"output":{"name":"bool"}}],[11,"clone","","",1,{"inputs":[{"name":"self"}],"output":{"name":"infotype"}}],[11,"fmt","","",2,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"eq","","",2,{"inputs":[{"name":"self"},{"name":"fetchorientation"}],"output":{"name":"bool"}}],[11,"clone","","",2,{"inputs":[{"name":"self"}],"output":{"name":"fetchorientation"}}],[11,"fmt","","",3,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"eq","","",3,{"inputs":[{"name":"self"},{"name":"environmentattribute"}],"output":{"name":"bool"}}],[11,"clone","","",3,{"inputs":[{"name":"self"}],"output":{"name":"environmentattribute"}}],[11,"fmt","","",4,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"eq","","",4,{"inputs":[{"name":"self"},{"name":"sqlcdatatype"}],"output":{"name":"bool"}}],[11,"clone","","",4,{"inputs":[{"name":"self"}],"output":{"name":"sqlcdatatype"}}],[11,"fmt","","",5,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"eq","","",5,{"inputs":[{"name":"self"},{"name":"inputoutput"}],"output":{"name":"bool"}}],[11,"clone","","",5,{"inputs":[{"name":"self"}],"output":{"name":"inputoutput"}}],[6,"SQLHANDLE","","",null,null],[6,"SQLHENV","","",null,null],[6,"SQLHDBC","","The connection handle references storage of all information about the connection to the data source, including status, transaction state, and error information.",null,null],[6,"SQLHSTMT","","",null,null],[6,"SQLSMALLINT","","",null,null],[6,"SQLUSMALLINT","","",null,null],[6,"SQLINTEGER","","",null,null],[6,"SQLUINTEGER","","",null,null],[6,"SQLPOINTER","","",null,null],[6,"SQLCHAR","","",null,null],[6,"SQLLEN","","",null,null],[6,"SQLULEN","","",null,null],[6,"SQLHWND","","",null,null],[17,"SQL_OV_ODBC2","","",null,null],[17,"SQL_OV_ODBC3","","",null,null],[17,"SQL_OV_ODBC3_80","","",null,null],[17,"SQL_NTS","","",null,null],[17,"SQL_NTSL","","",null,null],[17,"SQL_MAX_MESSAGE_LENGTH","","Maximum message length",null,null],[17,"SQL_SQLSTATE_SIZE","","",null,null],[17,"SQL_NULL_DATA","","",null,null],[17,"SQL_NO_TOTAL","","",null,null],[11,"fmt","","",6,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"eq","","",6,{"inputs":[{"name":"self"},{"name":"sqldatatype"}],"output":{"name":"bool"}}],[11,"clone","","",6,{"inputs":[{"name":"self"}],"output":{"name":"sqldatatype"}}],[11,"fmt","","",7,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"eq","","",7,{"inputs":[{"name":"self"},{"name":"handletype"}],"output":{"name":"bool"}}],[11,"clone","","",7,{"inputs":[{"name":"self"}],"output":{"name":"handletype"}}],[11,"fmt","","",8,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"eq","","",8,{"inputs":[{"name":"self"},{"name":"sqldriverconnectoption"}],"output":{"name":"bool"}}],[11,"clone","","",8,{"inputs":[{"name":"self"}],"output":{"name":"sqldriverconnectoption"}}]],"paths":[[4,"SQLRETURN"],[4,"InfoType"],[4,"FetchOrientation"],[4,"EnvironmentAttribute"],[4,"SqlCDataType"],[4,"InputOutput"],[4,"SqlDataType"],[4,"HandleType"],[4,"SqlDriverConnectOption"]]};
initSearch(searchIndex);
