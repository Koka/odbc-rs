use super::GenericError;
use odbc_safe::{ResultSet as RS, Unprepared, DataType, sys, ReturnOption, Statement, Positioned, Indicator};
use std::ops::{Deref, DerefMut};
use std::error::Error;
use std::iter::Iterator;
use std::mem;

#[derive(Debug)]
pub struct ResultSet<'con, 'p, 'col>(RS<'con, 'p, 'col, Unprepared>);

#[derive(Debug)]
pub struct RowIterator<'con, 'p, 'col>(Option<RS<'con, 'p, 'col, Unprepared>>, Option<Statement<'con, 'p, 'col, Positioned>>, u16);

pub struct Row(u16);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ColumnDescription {
    pub name: Vec<u8>,
    pub data_type: Option<DataType>,
    pub nullable: Option<bool>,
}

impl<'con, 'p, 'col> ResultSet<'con, 'p, 'col> {
    pub fn new(rs: RS<'con, 'p, 'col, Unprepared>) -> Self {
        ResultSet(rs)
    }

    pub fn into_raw(self) -> RS<'con, 'p, 'col, Unprepared> {
        self.0
    }
}

impl<'con, 'p, 'col> Deref for ResultSet<'con, 'p, 'col> {
    type Target = RS<'con, 'p, 'col, Unprepared>;

    fn deref(&self) -> &<Self as Deref>::Target {
        &self.0
    }
}

impl<'con, 'p, 'col> DerefMut for ResultSet<'con, 'p, 'col> {
    fn deref_mut(&mut self) -> &mut RS<'con, 'p, 'col, Unprepared> {
        &mut self.0
    }
}

impl<'con, 'p, 'col> ResultSet<'con, 'p, 'col> {
    pub fn column_count(&self) -> Result<u16, Box<Error>> {
        self.num_result_cols()
            .map_error(|_err| GenericError("Unable to count columns".to_owned()))
            .success()
            .map({ |num| num as u16 })
    }

    pub fn describe_columns(&mut self) -> Result<Vec<ColumnDescription>, Box<Error>> {
        let cols = self.column_count()?;
        let mut result: Vec<ColumnDescription> = Vec::with_capacity(cols as usize);

        let mut buffer = [0u8; 512];
        let mut ind: sys::SQLSMALLINT = 0;
        let mut nul = sys::Nullable::SQL_NULLABLE_UNKNOWN;

        for index in 1..(cols + 1) {
            let col: Result<Option<DataType>, Box<Error>> = self.describe_col(index, &mut buffer[..], &mut ind, &mut nul)
                .map_error(|_| GenericError("Error getting column info".to_owned())).success();

            result.push(ColumnDescription {
                data_type: col?,
                name: buffer[0..ind as usize].to_owned(),
                nullable: match nul {
                    sys::Nullable::SQL_NULLABLE_UNKNOWN => None,
                    sys::Nullable::SQL_NULLABLE => Some(true),
                    sys::Nullable::SQL_NO_NULLS => Some(false),
                },
            });
        }

        Ok(result)
    }

    pub fn rows(self) -> Result<RowIterator<'con, 'p, 'col>, Box<Error>> {
        let cols: Result<u16, GenericError> = self.0.num_result_cols()
            .map_error(|_err| GenericError("Unable to count columns".to_owned()))
            .success()
            .map({ |num| num as u16 });

        let cols = cols?;

        Ok(RowIterator(Some(self.0), None, cols))
    }
}

impl Row {
    pub fn length(&self) -> u16 {
        self.0
    }

    pub fn get_col(&mut self, index: u16) -> Result<Option<Vec<u8>>, Box<Error>> {
        Ok(None)
    }
}

impl<'con, 'p, 'col> RowIterator<'con, 'p, 'col> {
    fn get_col(&mut self, index: u16) -> Result<Option<Vec<u8>>, Box<Error>> {
        match self.1 {
            None => Ok(None),
            Some(ref mut cursor) => {
                let mut buffer = [0u8; 1024];
                match cursor.get_data(index, &mut buffer as &mut [u8]) {
                    ReturnOption::Success(ind) | ReturnOption::Info(ind) => {
                        match ind {
                            Indicator::NoTotal => Err(Box::new(GenericError("No total!".to_owned()))),
                            Indicator::Null => Ok(None),
                            Indicator::Length(l) => Ok(Some(buffer[0..l as usize].to_owned()))
                        }
                    }
                    ReturnOption::NoData(_) => Err(Box::new(GenericError("No field data".to_owned()))),
                    ReturnOption::Error(_) => Err(Box::new(GenericError("Error fetching field data".to_owned()))),
                }
            }
        }

    }
}

impl<'con, 'p, 'col> Iterator for RowIterator<'con, 'p, 'col> {
    type Item = Row;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        if let None = self.1 {
            if let Some(rs) = mem::replace(&mut self.0, None) {
                self.1 = match rs.fetch() {
                    ReturnOption::Success(r) | ReturnOption::Info(r) => Some(r),
                    ReturnOption::NoData(_) => None,
                    ReturnOption::Error(_e) => None //TODO - error
                };
            }
        }

        let ret_value = match self.1 {
            None => None,
            Some(_) => {
                Some(Row(self))
            }
        };

        let new_cursor = match mem::replace(&mut self.1, None) {
            None => None,
            Some(cursor) => match cursor.fetch() {
                ReturnOption::Success(r) | ReturnOption::Info(r) => Some(r),
                ReturnOption::NoData(_) => None,
                ReturnOption::Error(_e) => None //TODO - error
            }
        };

        self.1 = new_cursor;

        ret_value
    }
}
