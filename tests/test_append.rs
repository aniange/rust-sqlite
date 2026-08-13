use rusqlite::Connection;
use rust_sqlite::functions::table::{sqlappendtable_impl, sqlcreatetable_impl};

#[test]
fn test_sqlappendtable_basic() {
    let conn = Connection::open_in_memory().unwrap();

    // 先创建表
    let create_data = vec![
        vec!["id".to_string(), "name".to_string()],
        vec!["1".to_string(), "Alice".to_string()],
    ];
    sqlcreatetable_impl(&conn, "users", create_data, None, None).unwrap();

    // 追加数据
    let append_data = vec![
        vec!["2".to_string(), "Bob".to_string()],
        vec!["3".to_string(), "Charlie".to_string()],
    ];
    let result = sqlappendtable_impl(&conn, "users", append_data);
    assert!(result.is_ok(), "{}", result.unwrap_err());
    assert!(result.unwrap().contains("2 rows appended"));

    // 验证总行数
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_sqlappendtable_table_not_exists() {
    let conn = Connection::open_in_memory().unwrap();
    let data = vec![vec!["1".to_string()]];
    let result = sqlappendtable_impl(&conn, "nonexistent", data);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn test_sqlappendtable_column_mismatch() {
    let conn = Connection::open_in_memory().unwrap();

    let create_data = vec![
        vec!["a".to_string(), "b".to_string()],
        vec!["1".to_string(), "2".to_string()],
    ];
    sqlcreatetable_impl(&conn, "t", create_data, None, None).unwrap();

    let append_data = vec![vec!["1".to_string(), "2".to_string(), "3".to_string()]];
    let result = sqlappendtable_impl(&conn, "t", append_data);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not match"));
}

#[test]
fn test_sqlappendtable_empty_data() {
    let conn = Connection::open_in_memory().unwrap();
    let create_data = vec![vec!["id".to_string()], vec!["1".to_string()]];
    sqlcreatetable_impl(&conn, "t", create_data, None, None).unwrap();

    let result = sqlappendtable_impl(&conn, "t", vec![]);
    assert!(result.is_err());
}
