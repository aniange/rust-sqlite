use rust_sqlite::conn::with_conn;
use rust_sqlite::functions::exec::sqlscript_impl;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_script_raw_sql_memory_db() {
    let script = r#"
        CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT);
        INSERT INTO users (name) VALUES ('Alice');
        INSERT INTO users (name) VALUES ('Bob');
        CREATE INDEX idx_users_name ON users(name);
    "#;

    let result = sqlscript_impl(rust_sqlite::conn::MEMORY_DB_URI, script);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Script executed successfully");

    let count = with_conn(rust_sqlite::conn::MEMORY_DB_URI, |conn| {
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM users").unwrap();
        let count: i64 = stmt.query_row([], |r| r.get(0)).unwrap();
        Ok(count)
    });
    assert_eq!(count.unwrap(), 2);
}

#[test]
fn test_script_from_utf8_file() {
    let mut file = NamedTempFile::with_suffix(".sql").unwrap();
    writeln!(
        file,
        "CREATE TABLE file_test (id INTEGER PRIMARY KEY, val TEXT);"
    )
    .unwrap();
    writeln!(file, "INSERT INTO file_test (val) VALUES ('from_file');").unwrap();
    let path = file.path().to_str().unwrap();

    let result = sqlscript_impl(rust_sqlite::conn::MEMORY_DB_URI, path);
    assert!(result.is_ok());

    let val = with_conn(rust_sqlite::conn::MEMORY_DB_URI, |conn| {
        let mut stmt = conn.prepare("SELECT val FROM file_test").unwrap();
        let val: String = stmt.query_row([], |r| r.get(0)).unwrap();
        Ok(val)
    });
    assert_eq!(val.unwrap(), "from_file");
}

#[test]
fn test_script_from_gbk_file() {
    let mut file = NamedTempFile::with_suffix(".sql").unwrap();
    // GBK encoded: CREATE TABLE gbk_table (name TEXT); INSERT INTO gbk_table VALUES ('中文');
    let gbk_bytes: Vec<u8> = vec![
        0x43, 0x52, 0x45, 0x41, 0x54, 0x45, 0x20, 0x54, 0x41, 0x42, 0x4c, 0x45, 0x20, 0x67, 0x62,
        0x6b, 0x5f, 0x74, 0x61, 0x62, 0x6c, 0x65, 0x20, 0x28, 0x6e, 0x61, 0x6d, 0x65, 0x20, 0x54,
        0x45, 0x58, 0x54, 0x29, 0x3b, 0x0a, 0x49, 0x4e, 0x53, 0x45, 0x52, 0x54, 0x20, 0x49, 0x4e,
        0x54, 0x4f, 0x20, 0x67, 0x62, 0x6b, 0x5f, 0x74, 0x61, 0x62, 0x6c, 0x65, 0x20, 0x56, 0x41,
        0x4c, 0x55, 0x45, 0x53, 0x20, 0x28, 0x27, 0xd6, 0xd0, 0xce, 0xc4, 0x27, 0x29, 0x3b, 0x0a,
    ];
    file.write_all(&gbk_bytes).unwrap();
    let path = file.path().to_str().unwrap();

    let result = sqlscript_impl(rust_sqlite::conn::MEMORY_DB_URI, path);
    assert!(result.is_ok());

    let val = with_conn(rust_sqlite::conn::MEMORY_DB_URI, |conn| {
        let mut stmt = conn.prepare("SELECT name FROM gbk_table").unwrap();
        let val: String = stmt.query_row([], |r| r.get(0)).unwrap();
        Ok(val)
    });
    assert_eq!(val.unwrap(), "中文");
}

#[test]
fn test_script_with_file_db() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let db_path = tmp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    let mut file = NamedTempFile::with_suffix(".sql").unwrap();
    writeln!(file, "CREATE TABLE persistent (id INTEGER PRIMARY KEY);").unwrap();
    writeln!(file, "INSERT INTO persistent VALUES (42);").unwrap();
    let script_path = file.path().to_str().unwrap();

    let result = sqlscript_impl(db_path_str, script_path);
    assert!(result.is_ok());

    let val = with_conn(db_path_str, |conn| {
        let mut stmt = conn.prepare("SELECT id FROM persistent").unwrap();
        let val: i64 = stmt.query_row([], |r| r.get(0)).unwrap();
        Ok(val)
    });
    assert_eq!(val.unwrap(), 42);
}

#[test]
fn test_script_invalid_sql_returns_error() {
    let script = "CREATE TABLE bad (id INTEGER); THIS IS NOT SQL;";
    let result = sqlscript_impl(rust_sqlite::conn::MEMORY_DB_URI, script);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Script execution failed"));
}

#[test]
fn test_script_nonexistent_path_treated_as_raw_sql() {
    let result = sqlscript_impl(
        rust_sqlite::conn::MEMORY_DB_URI,
        "/definitely/not/a/real/file.sql",
    );
    assert!(result.is_err());
}

#[test]
fn test_script_multiple_ddl_and_dml() {
    let script = r#"
        CREATE TABLE orders (
            id INTEGER PRIMARY KEY,
            product TEXT,
            amount REAL
        );
        CREATE INDEX idx_product ON orders(product);
        INSERT INTO orders (product, amount) VALUES ('Apple', 19.99);
        INSERT INTO orders (product, amount) VALUES ('Banana', 9.50);
        UPDATE orders SET amount = 20.00 WHERE product = 'Apple';
    "#;

    let result = sqlscript_impl(rust_sqlite::conn::MEMORY_DB_URI, script);
    assert!(result.is_ok());

    let (count, total) = with_conn(rust_sqlite::conn::MEMORY_DB_URI, |conn| {
        let mut stmt = conn
            .prepare("SELECT COUNT(*), SUM(amount) FROM orders")
            .unwrap();
        let (count, total): (i64, f64) =
            stmt.query_row([], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        Ok((count, total))
    })
    .unwrap();

    assert_eq!(count, 2);
    assert!((total - 29.50).abs() < 0.001);
}
