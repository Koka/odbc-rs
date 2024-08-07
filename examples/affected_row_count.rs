//! Shows affected row count

extern crate odbc;
extern crate odbc_safe;

use odbc::*;
use odbc_safe::AutocommitOn;

fn main() {
    let env = create_environment_v3().unwrap();
    let conn = env.connect("TestDataSource", "", "").unwrap();
    exec(
        &conn,
        "INSERT INTO movies (title, year) VALUES ('TEST movie', 9999), ('TEST movie', 9998)",
    );
    exec(&conn, "DELETE FROM movies WHERE title = 'TEST movie'");
}

fn exec(conn: &Connection<AutocommitOn>, sql: &str) {
    let stmt = Statement::with_parent(conn).unwrap();
    let rs = stmt.exec_direct(sql).unwrap();
    match rs {
        Data(stmt) => {
            let row_count = stmt.affected_row_count().unwrap();
            println!(
                "Has data and affected row count for last statement: {:?}",
                row_count
            );
        }
        NoData(stmt) => {
            let row_count = stmt.affected_row_count().unwrap();
            println!(
                "No data and affected row count for last statement: {:?}",
                row_count
            );
        }
    }
}
