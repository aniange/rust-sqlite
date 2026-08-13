use rusqlite::Connection;
use rust_sqlite::functions::csv_import::sqlimportcsv_impl;
use std::io::Write;

#[test]
fn test_sqlimportcsv_impl_basic() {
    let conn = Connection::open_in_memory().unwrap();
    let mut temp = tempfile::NamedTempFile::new().unwrap();
    writeln!(temp, "id,name,score").unwrap();
    writeln!(temp, "1,Alice,95.5").unwrap();
    writeln!(temp, "2,Bob,88.0").unwrap();
    let path = temp.path().to_str().unwrap();

    let result = sqlimportcsv_impl(&conn, path, "students", true, b',', None, None);
    assert!(result.is_ok(), "{}", result.unwrap_err());

    let mut stmt = conn.prepare("SELECT * FROM students").unwrap();
    let rows: Vec<_> = stmt.query_map([], |r| {
        Ok((r.get::<_, i64>(0).unwrap(), r.get::<_, String>(1).unwrap(), r.get::<_, f64>(2).unwrap()))
    }).unwrap().collect();
    assert_eq!(rows.len(), 2);
}

#[test]
fn test_sqlimportcsv_impl_no_header() {
    let conn = Connection::open_in_memory().unwrap();
    let mut temp = tempfile::NamedTempFile::new().unwrap();
    writeln!(temp, "1,Alice").unwrap();
    writeln!(temp, "2,Bob").unwrap();
    let path = temp.path().to_str().unwrap();

    let result = sqlimportcsv_impl(&conn, path, "noheader", false, b',', None, None);
    assert!(result.is_ok());

    let stmt = conn.prepare("SELECT * FROM noheader").unwrap();
    let col_count = stmt.column_count();
    assert_eq!(col_count, 2);
}

#[test]
fn test_sqlimportcsv_impl_tab_delimiter() {
    let conn = Connection::open_in_memory().unwrap();
    let mut temp = tempfile::NamedTempFile::new().unwrap();
    writeln!(temp, "id\tname").unwrap();
    writeln!(temp, "1\tAlice").unwrap();
    let path = temp.path().to_str().unwrap();

    let result = sqlimportcsv_impl(&conn, path, "tsv_table", true, b'\t', None, None);
    assert!(result.is_ok());

    let mut stmt = conn.prepare("SELECT name FROM tsv_table WHERE id = 1").unwrap();
    let name: String = stmt.query_row([], |r| r.get(0)).unwrap();
    assert_eq!(name, "Alice");
}

#[test]
fn test_sqlimportcsv_impl_gbk_encoding() {
    let conn = Connection::open_in_memory().unwrap();
    let mut temp = tempfile::NamedTempFile::new().unwrap();
    let (gbk_bytes, _, _) = encoding_rs::GB18030.encode("name,score\nAlice,90\nBob,85");
    temp.write_all(&gbk_bytes).unwrap();
    let path = temp.path().to_str().unwrap();

    let result = sqlimportcsv_impl(&conn, path, "gbk_table", true, b',', None, None);
    assert!(result.is_ok(), "{}", result.unwrap_err());

    let mut stmt = conn.prepare("SELECT * FROM gbk_table").unwrap();
    let rows: Vec<String> = stmt.query_map([], |r| {
        r.get::<_, String>(0)
    }).unwrap().map(|r| r.unwrap()).collect();
    assert_eq!(rows[0], "Alice");
    assert_eq!(rows[1], "Bob");
}

#[test]
fn test_sqlimportcsv_impl_empty_csv() {
    let conn = Connection::open_in_memory().unwrap();
    let temp = tempfile::NamedTempFile::new().unwrap();
    let path = temp.path().to_str().unwrap();

    let result = sqlimportcsv_impl(&conn, path, "empty_csv", true, b',', None, None);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("empty") || err.contains("CSV"));
}

#[test]
fn test_sqlimportcsv_impl_custom_columns_and_types() {
    let conn = Connection::open_in_memory().unwrap();
    let mut temp = tempfile::NamedTempFile::new().unwrap();
    writeln!(temp, "a,b,c").unwrap();
    writeln!(temp, "1,hello,3.14").unwrap();
    let path = temp.path().to_str().unwrap();

    let cols = Some(vec!["x".to_string(), "y".to_string(), "z".to_string()]);
    let types = Some(vec!["INTEGER".to_string(), "TEXT".to_string(), "REAL".to_string()]);
    let result = sqlimportcsv_impl(&conn, path, "custom", true, b',', cols, types);
    assert!(result.is_ok());

    let mut stmt = conn.prepare("PRAGMA table_info(custom)").unwrap();
    let info: Vec<(String, String)> = stmt.query_map([], |r| {
        Ok((r.get::<_, String>(1).unwrap(), r.get::<_, String>(2).unwrap()))
    }).unwrap().map(|x| x.unwrap()).collect();
    assert_eq!(info, vec![
        ("x".to_string(), "INTEGER".to_string()),
        ("y".to_string(), "TEXT".to_string()),
        ("z".to_string(), "REAL".to_string()),
    ]);
}
