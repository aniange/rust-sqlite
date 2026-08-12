use rust_sqlite::functions::exec::sqlexec_impl;

fn temp_db_path(prefix: &str) -> String {
    let mut path = std::env::temp_dir();
    path.push(format!("{}_{}.db", prefix, std::process::id()));
    path.to_string_lossy().to_string()
}

#[test]
fn test_sqlexec_impl_create_table() {
    let db_path = temp_db_path("test_exec_create");
    let _ = std::fs::remove_file(&db_path);

    let result = sqlexec_impl(&db_path, "CREATE TABLE t (id INTEGER)");
    assert!(result.is_ok());

    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn test_sqlexec_impl_insert_and_affected_rows() {
    let db_path = temp_db_path("test_exec_insert");
    let _ = std::fs::remove_file(&db_path);

    sqlexec_impl(&db_path, "CREATE TABLE t (id INTEGER, name TEXT)").unwrap();
    let result = sqlexec_impl(&db_path, "INSERT INTO t VALUES (1, 'Alice'), (2, 'Bob')");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 2);

    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn test_sqlexec_impl_invalid_sql() {
    let db_path = temp_db_path("test_exec_invalid");
    let _ = std::fs::remove_file(&db_path);

    let result = sqlexec_impl(&db_path, "INVALID SQL");
    assert!(result.is_err());

    let _ = std::fs::remove_file(&db_path);
}
