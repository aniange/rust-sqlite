use rust_sqlite::functions::metadata::sqlencrypt_impl;

fn temp_db_path(prefix: &str) -> String {
    let mut path = std::env::temp_dir();
    path.push(format!("{}_{}.db", prefix, std::process::id()));
    path.to_string_lossy().to_string()
}

/// 验证加密库可用（表存在）
fn verify_encrypted_db(path: &str, password: &str, expected_table: &str) {
    let conn = rusqlite::Connection::open(path).unwrap();
    let safe_pwd = password.replace('\'', "''");
    conn.execute_batch(&format!("PRAGMA key = '{}';", safe_pwd))
        .unwrap();
    let count: i64 = conn
        .query_row(
            &format!(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='{}'",
                expected_table
            ),
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        count, 1,
        "Table {} should exist in encrypted db",
        expected_table
    );
}

/// 验证明文库可用（表存在）
fn verify_plain_db(path: &str, expected_table: &str) {
    let conn = rusqlite::Connection::open(path).unwrap();
    let count: i64 = conn
        .query_row(
            &format!(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='{}'",
                expected_table
            ),
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        count, 1,
        "Table {} should exist in plain db",
        expected_table
    );
}

#[test]
fn test_encrypt_plain_to_encrypted() {
    let source = temp_db_path("test_enc_src");
    let target = temp_db_path("test_enc_tgt");
    let _ = std::fs::remove_file(&source);
    let _ = std::fs::remove_file(&target);

    // 创建明文源库
    {
        let conn = rusqlite::Connection::open(&source).unwrap();
        conn.execute("CREATE TABLE t1 (id INTEGER, name TEXT)", [])
            .unwrap();
        conn.execute("INSERT INTO t1 VALUES (1, 'Alice')", [])
            .unwrap();
    }

    // 加密导出
    let result = sqlencrypt_impl(&source, &target, Some("mykey"), None);
    assert!(result.is_ok(), "{}", result.unwrap_err());
    assert!(result.unwrap().contains("Encrypted"));

    // 验证加密库可用
    verify_encrypted_db(&target, "mykey", "t1");
    // 验证源库不受影响
    verify_plain_db(&source, "t1");

    let _ = std::fs::remove_file(&source);
    let _ = std::fs::remove_file(&target);
}

#[test]
fn test_encrypt_encrypted_to_plain() {
    let source = temp_db_path("test_dec_src");
    let target = temp_db_path("test_dec_tgt");
    let _ = std::fs::remove_file(&source);
    let _ = std::fs::remove_file(&target);

    // 创建加密源库
    {
        let conn = rusqlite::Connection::open(&source).unwrap();
        conn.execute_batch("PRAGMA key = 'secret';").unwrap();
        conn.execute("CREATE TABLE t2 (id INTEGER)", []).unwrap();
    }

    // 解密导出（空密码表示明文）
    let result = sqlencrypt_impl(&source, &target, Some(""), Some("secret"));
    assert!(result.is_ok(), "{}", result.unwrap_err());
    assert!(result.unwrap().contains("Decrypted"));

    // 验证明文库可用
    verify_plain_db(&target, "t2");

    let _ = std::fs::remove_file(&source);
    let _ = std::fs::remove_file(&target);
}

#[test]
fn test_encrypt_change_password() {
    let source = temp_db_path("test_chpwd_src");
    let target = temp_db_path("test_chpwd_tgt");
    let _ = std::fs::remove_file(&source);
    let _ = std::fs::remove_file(&target);

    // 创建加密源库
    {
        let conn = rusqlite::Connection::open(&source).unwrap();
        conn.execute_batch("PRAGMA key = 'oldpass';").unwrap();
        conn.execute("CREATE TABLE t3 (val TEXT)", []).unwrap();
    }

    // 更换密码加密导出
    let result = sqlencrypt_impl(&source, &target, Some("newpass"), Some("oldpass"));
    assert!(result.is_ok(), "{}", result.unwrap_err());
    assert!(result.unwrap().contains("Encrypted"));

    // 验证新密码可打开
    verify_encrypted_db(&target, "newpass", "t3");
    // 验证旧密码打不开
    let conn = rusqlite::Connection::open(&target).unwrap();
    conn.execute_batch("PRAGMA key = 'oldpass';").unwrap();
    let res = conn.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()));
    assert!(res.is_err(), "Old password should not work");

    let _ = std::fs::remove_file(&source);
    let _ = std::fs::remove_file(&target);
}

#[test]
fn test_encrypt_target_exists() {
    let source = temp_db_path("test_exist_src");
    let target = temp_db_path("test_exist_tgt");
    let _ = std::fs::remove_file(&source);
    let _ = std::fs::remove_file(&target);

    // 让目标文件已存在
    let _ = rusqlite::Connection::open(&source).unwrap();
    let _ = rusqlite::Connection::open(&target).unwrap();

    let result = sqlencrypt_impl(&source, &target, Some("key"), None);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already exists"));

    let _ = std::fs::remove_file(&source);
    let _ = std::fs::remove_file(&target);
}

#[test]
fn test_encrypt_same_path() {
    let path = temp_db_path("test_same");
    let _ = std::fs::remove_file(&path);
    let _ = rusqlite::Connection::open(&path).unwrap();

    let result = sqlencrypt_impl(&path, &path, Some("key"), None);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("same file"));

    let _ = std::fs::remove_file(&path);
}
