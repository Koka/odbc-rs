use odbc_safe::{Connection as Conn, Statement as Stmt, ReturnOption};
use std::ops::{Deref, DerefMut};
use std::error::Error;
use super::{GenericError, SqlString};

mod result_set;
pub use self::result_set::ResultSet;

pub struct Connection<'c>(Conn<'c>);

impl <'c> Connection<'c> {
    pub fn new(conn: Conn<'c>) -> Self {
        Connection(conn)
    }

    pub fn into_raw(self) -> Conn<'c> {
        self.0
    }

    pub fn select<S: Into<String>, F, T>(&self, sql_str: S, closure: F) -> Result<T, Box<Error>>
        where F: FnOnce(Option<ResultSet>) -> Result<T, Box<Error>>
    {
        let stmt: Result<Stmt, Box<Error>> = Stmt::with_parent(self)
            .map_error(|_| GenericError("Error allocating statement".to_owned()))
            .success();

        let converted_string = SqlString::from(sql_str)?;
        let ropt = stmt?.exec_direct(&converted_string);

        match ropt {
            ReturnOption::Success(s) | ReturnOption::Info(s) => closure(Some(ResultSet::new(s))),
            ReturnOption::NoData(_) => closure(None),
            ReturnOption::Error(_e) => Err(Box::new(GenericError("Error executing statement".to_owned()))),
        }
    }
}

impl <'c> Deref for Connection<'c> {
    type Target = Conn<'c>;

    fn deref(&self) -> &<Self as Deref>::Target {
        &self.0
    }
}

impl <'c> DerefMut for Connection<'c> {
    fn deref_mut(&mut self) -> &mut Conn<'c> {
        &mut self.0
    }
}