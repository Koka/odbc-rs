//! Contains test for all types supporting `bind_parameter`
//!
//! These tests assume there is a Stage table with a Varchar in 'A', an Integer in 'B' and a Real
//! in 'C'
extern crate odbc;
use odbc::*;

const A: &'static str = "SELECT A FROM TEST_TYPES WHERE A = ?;";
const B: &'static str = "SELECT B FROM TEST_TYPES WHERE B = ?;";
const C: &'static str = "SELECT C FROM TEST_TYPES WHERE C = ?;";

macro_rules! test_type {
    ($t:ty, $c:expr, $e:expr) => ({
        let param: $t = $e;

        let env = Environment::new().unwrap().set_odbc_version_3().unwrap();
        let conn = DataSource::with_parent(&env).unwrap().connect("TestDataSource", "", "").unwrap();
        let stmt = Statement::with_parent(&conn).unwrap();
        let stmt = stmt.bind_parameter(1, &param).unwrap();
        if let Ok(Data(mut cursor)) = stmt.exec_direct($c){
            if let Some(mut row) = cursor.fetch().unwrap(){
                let value : $t = row.get_data(1).unwrap().unwrap();
                assert_eq!(value, $e);
            } else{
                panic!("Result set has been empty");
            }
        }else{
            panic!("SELECT did not return result set");
        }
    })
}

#[test]
fn _ref_str() {
    test_type!(&str, A, "Hello, World!")
}

#[test]
fn _string() {
    test_type!(String, A, String::from("Hello, World!"))
}