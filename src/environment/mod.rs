
use std::error::Error;
use super::{GenericError, safe, SqlString, Connection};
use std::sync::Mutex;
use odbc_safe::{Environment, Odbc3, sys, DataSource, Connection as Conn};
use std::ops::{Deref, DerefMut};

mod list_data_sources;

pub struct Env(Environment<Odbc3>);

unsafe impl Send for Env {}

lazy_static! {
    static ref ODBC_ENV: Mutex<Result<Env, GenericError>> = {
        Mutex::new(
            Environment::new().map_error(|_e| GenericError("Failed to obtain ODBC environment".to_owned()))
                .success()
                .and_then(
                    |env| env.declare_version_3().map_error(|_env| GenericError("Failed to obtain ODBC v3 environment".to_owned())).success()
                ).map(
                    |env| Env(env)
                )
        )
    };
}

impl Deref for Env {
    type Target = Environment<Odbc3>;

    fn deref(&self) -> &<Self as Deref>::Target {
        &self.0
    }
}

impl DerefMut for Env {
    fn deref_mut(&mut self) -> &mut Environment<Odbc3> {
        &mut self.0
    }
}

impl Env {
    pub fn with_env<F, T>(closure: F) -> Result<T, Box<Error>>
        where F: FnOnce(&Environment<Odbc3>) -> Result<T, Box<Error>>
    {
        let res = ODBC_ENV.lock()?;
        match *res {
            Ok(ref env) => closure(&env),
            Err(ref e) => Err(e.clone().into())
        }
    }

    pub fn with_env_mut<F, T>(closure: F) -> Result<T, Box<Error>>
        where F: FnOnce(&mut Environment<Odbc3>) -> Result<T, Box<Error>>
    {
        let mut res = ODBC_ENV.lock()?;
        match *res {
            Ok(ref mut env) => closure(env),
            Err(ref e) => Err(e.clone().into())
        }
    }

    pub fn with_connection<S: Into<String>, F, T>(connection_string: S, closure: F) -> Result<T, Box<Error>>
        where F: FnOnce(Connection) -> Result<T, Box<Error>>
    {
        let converted_string = SqlString::from(connection_string)?;

        let res = ODBC_ENV.lock()?;
        match *res {
            Ok(ref env) => {
                let conn: Result<Conn, Box<Error>> = DataSource::with_parent(&env.0)
                    .map_error(|_err| GenericError("Unable to create connection".to_owned()))
                    .success()
                    .and_then(
                        |ds| ds.connect_with_connection_string(&converted_string)
                            .map_error(|_err| GenericError("Unable to connect to database".to_owned()))
                            .success()
                    );

                let mut conn = Connection::new(conn?);

                closure(conn)
            },
            Err(ref e) => Err(e.clone().into())
        }
    }
}
