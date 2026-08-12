use rust_sqlite::functions::csv_export::sqlexportcsv_impl;
use std::io::Read;

#[test]
fn test_sqlexportcsv_basic() {
    let db_path = format!("test_export_{}.db", std::process::id());
    let csv_path = format!("test_export_{}.csv", std::process::id());
    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(&csv_path);

    let conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.execute("CREATE TABLE t (id INTEGER, name TEXT)", []).unwrap();
    conn.execute("INSERT INTO t VALUES (1, 'Alice'), (2, 'Bob')", []).unwrap();
    drop(conn);

    let result = sqlexportcsv_impl(&db_path, "SELECT * FROM t", &csv_path, b',');
    assert!(result.is_ok(), "{}", result.unwrap_err());
    assert!(result.unwrap().contains("2 rows"));

    // 验证文件内容
    let mut content = String::new();
    std::fs::File::open(&csv_path).unwrap().read_to_string(&mut content).unwrap();
    assert!(content.contains("id,name"));
    assert!(content.contains("1,Alice"));

    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(&csv_path);
}

#[test]
fn test_sqlexportcsv_tab_delimiter() {
    let db_path = format!("test_export_tsv_{}.db", std::process::id());
    let csv_path = format!("test_export_tsv_{}.csv", std::process::id());
    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(&csv_path);

    let conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.execute("CREATE TABLE t (id INTEGER)", []).unwrap();
    conn.execute("INSERT INTO t VALUES (1)", []).unwrap();
    drop(conn);

    let result = sqlexportcsv_impl(&db_path, "SELECT * FROM t", &csv_path, b'\t');
    assert!(result.is_ok());

    let mut content = String::new();
    std::fs::File::open(&csv_path).unwrap().read_to_string(&mut content).unwrap();
    assert!(content.contains("id\t"));

    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(&csv_path);
}
