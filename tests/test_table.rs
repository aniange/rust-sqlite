use rusqlite::Connection;
use rust_sqlite::functions::table::sqlcreatetable_impl;

#[test]
fn test_sqlcreatetable_impl_basic() {
    let conn = Connection::open_in_memory().unwrap();
    let data = vec![
        vec!["id".to_string(), "name".to_string()],
        vec!["1".to_string(), "Alice".to_string()],
        vec!["2".to_string(), "Bob".to_string()],
    ];
    let result = sqlcreatetable_impl(&conn, "users", data, None, None);
    assert!(result.is_ok(), "{}", result.unwrap_err());
    assert!(result.unwrap().contains("users"));

    let mut stmt = conn.prepare("SELECT * FROM users").unwrap();
    let rows: Vec<_> = stmt
        .query_map([], |r| {
            Ok((r.get::<_, i64>(0).unwrap(), r.get::<_, String>(1).unwrap()))
        })
        .unwrap()
        .collect();
    assert_eq!(rows.len(), 2);
}

#[test]
fn test_sqlcreatetable_impl_explicit_columns() {
    let conn = Connection::open_in_memory().unwrap();
    let data = vec![
        vec!["1".to_string(), "Alice".to_string()],
        vec!["2".to_string(), "Bob".to_string()],
    ];
    let cols = Some(vec!["uid".to_string(), "uname".to_string()]);
    let result = sqlcreatetable_impl(&conn, "users2", data, cols, None);
    assert!(result.is_ok());

    let mut stmt = conn.prepare("PRAGMA table_info(users2)").unwrap();
    let col_names: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(1))
        .unwrap()
        .map(|x| x.unwrap())
        .collect();
    assert_eq!(col_names, vec!["uid", "uname"]);
}

#[test]
fn test_sqlcreatetable_impl_explicit_types() {
    let conn = Connection::open_in_memory().unwrap();
    let data = vec![
        vec!["a".to_string(), "b".to_string()],
        vec!["1".to_string(), "3.14".to_string()],
    ];
    let cols = Some(vec!["x".to_string(), "y".to_string()]);
    let types = Some(vec!["INTEGER".to_string(), "REAL".to_string()]);
    let result = sqlcreatetable_impl(&conn, "typed_table", data, cols, types);
    assert!(result.is_ok());

    let mut stmt = conn.prepare("PRAGMA table_info(typed_table)").unwrap();
    let col_types: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(2))
        .unwrap()
        .map(|x| x.unwrap())
        .collect();
    assert_eq!(col_types, vec!["INTEGER", "REAL"]);
}

#[test]
fn test_sqlcreatetable_impl_empty_data() {
    let conn = Connection::open_in_memory().unwrap();
    let data: Vec<Vec<String>> = vec![];
    let result = sqlcreatetable_impl(&conn, "empty", data, None, None);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("empty"));
}

#[test]
fn test_sqlcreatetable_impl_idempotent() {
    let conn = Connection::open_in_memory().unwrap();
    let data = vec![vec!["id".to_string()], vec!["1".to_string()]];
    let _ = sqlcreatetable_impl(&conn, "idempotent", data.clone(), None, None).unwrap();
    let result2 = sqlcreatetable_impl(&conn, "idempotent", data, None, None);
    assert!(result2.is_ok());
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM idempotent", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_sqlcreatetable_impl_auto_infer_types() {
    let conn = Connection::open_in_memory().unwrap();
    let data = vec![
        vec!["id".to_string(), "name".to_string(), "score".to_string()],
        vec!["1".to_string(), "Alice".to_string(), "95.5".to_string()],
        vec!["2".to_string(), "Bob".to_string(), "88".to_string()],
    ];
    let result = sqlcreatetable_impl(&conn, "auto_types", data, None, None);
    assert!(result.is_ok());

    let mut stmt = conn.prepare("PRAGMA table_info(auto_types)").unwrap();
    let col_types: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(2))
        .unwrap()
        .map(|x| x.unwrap())
        .collect();
    assert_eq!(col_types, vec!["INTEGER", "TEXT", "REAL"]);
}
