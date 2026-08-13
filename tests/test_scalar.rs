use rust_sqlite::functions::query::sqlqueryscalar_impl;

#[test]
fn test_sqlqueryscalar_count() {
    let result = sqlqueryscalar_impl(":memory:", "SELECT 42");
    assert!(result.is_ok());
}

#[test]
fn test_sqlqueryscalar_text() {
    let result = sqlqueryscalar_impl(":memory:", "SELECT 'hello'");
    assert!(result.is_ok());
}

#[test]
fn test_sqlqueryscalar_from_table() {
    let db_path = format!("test_scalar_{}.db", std::process::id());
    let _ = std::fs::remove_file(&db_path);

    let conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.execute("CREATE TABLE t (id INTEGER, name TEXT)", []).unwrap();
    conn.execute("INSERT INTO t VALUES (1, 'Alice'), (2, 'Bob')", []).unwrap();
    drop(conn);

    let result = sqlqueryscalar_impl(&db_path, "SELECT name FROM t WHERE id = 1");
    assert!(result.is_ok());

    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn test_sqlqueryscalar_no_rows() {
    let result = sqlqueryscalar_impl(":memory:", "SELECT 1 WHERE 1=0");
    assert!(result.is_err());
}
