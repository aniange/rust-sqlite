use rust_sqlite::functions::exec::{sqlexec_impl, sqlbegin_impl, sqlcommit_impl, sqlrollback_impl};

#[test]
fn test_transaction_begin_commit() {
    let db_path = format!("test_tx_commit_{}.db", std::process::id());
    let _ = std::fs::remove_file(&db_path);

    // 先创建表
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.execute("CREATE TABLE t (id INTEGER)", []).unwrap();
    drop(conn);

    let result = sqlbegin_impl(&db_path);
    assert!(result.is_ok());

    // 在事务中插入数据（使用 sqlexec_impl，复用缓存连接）
    let result = sqlexec_impl(&db_path, "INSERT INTO t VALUES (1)");
    assert!(result.is_ok());

    let result = sqlcommit_impl(&db_path);
    assert!(result.is_ok());

    // 验证数据已提交
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM t", [], |r| r.get(0)).unwrap();
    assert_eq!(count, 1);
    drop(conn);

    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn test_transaction_begin_rollback() {
    let db_path = format!("test_tx_rollback_{}.db", std::process::id());
    let _ = std::fs::remove_file(&db_path);

    let conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.execute("CREATE TABLE t (id INTEGER)", []).unwrap();
    conn.execute("INSERT INTO t VALUES (1)", []).unwrap();
    drop(conn);

    let result = sqlbegin_impl(&db_path);
    assert!(result.is_ok());

    // 在事务中插入更多数据（使用 sqlexec_impl，复用缓存连接）
    let result = sqlexec_impl(&db_path, "INSERT INTO t VALUES (2)");
    assert!(result.is_ok());

    let result = sqlrollback_impl(&db_path);
    assert!(result.is_ok());

    // 验证数据已回滚，只有事务前的 1 条
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM t", [], |r| r.get(0)).unwrap();
    assert_eq!(count, 1);
    drop(conn);

    let _ = std::fs::remove_file(&db_path);
}
