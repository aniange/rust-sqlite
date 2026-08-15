use rust_sqlite::conn::with_conn;
use rust_sqlite::functions::csv_import::sqlimportcsvdir_impl;
use std::fs;
use std::io::Write;
use tempfile::TempDir;

fn create_test_csv(dir: &std::path::Path, name: &str, content: &str) {
    let path = dir.join(name);
    let mut file = fs::File::create(&path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
}

#[test]
fn test_import_empty_dir() {
    let tmp_dir = TempDir::new().unwrap();
    let result = sqlimportcsvdir_impl(
        &rusqlite::Connection::open_in_memory().unwrap(),
        tmp_dir.path().to_str().unwrap(),
        true,
        b',',
        None,
        None,
    );
    assert!(result.is_ok());
    assert!(result.unwrap().contains("Imported 0 CSV file(s)"));
}

#[test]
fn test_import_single_csv() {
    let tmp_dir = TempDir::new().unwrap();
    create_test_csv(tmp_dir.path(), "users.csv", "id,name\n1,Alice\n2,Bob\n");

    let result = sqlimportcsvdir_impl(
        &rusqlite::Connection::open_in_memory().unwrap(),
        tmp_dir.path().to_str().unwrap(),
        true,
        b',',
        None,
        None,
    );
    assert!(result.is_ok());
    let msg = result.unwrap();
    assert!(msg.contains("Imported 1 CSV file(s)"));
    assert!(msg.contains("users"));
}

#[test]
fn test_import_multiple_csv() {
    let tmp_dir = TempDir::new().unwrap();
    create_test_csv(tmp_dir.path(), "orders.csv", "id,product\n1,Apple\n");
    create_test_csv(tmp_dir.path(), "items.csv", "id,item\n1,Book\n");

    let result = sqlimportcsvdir_impl(
        &rusqlite::Connection::open_in_memory().unwrap(),
        tmp_dir.path().to_str().unwrap(),
        true,
        b',',
        None,
        None,
    );
    assert!(result.is_ok());
    let msg = result.unwrap();
    assert!(msg.contains("Imported 2 CSV file(s)"));
    assert!(msg.contains("orders"));
    assert!(msg.contains("items"));
}

#[test]
fn test_import_skips_non_csv() {
    let tmp_dir = TempDir::new().unwrap();
    create_test_csv(tmp_dir.path(), "data.csv", "a,b\n1,2\n");
    create_test_csv(tmp_dir.path(), "readme.txt", "This is not a CSV");

    let result = sqlimportcsvdir_impl(
        &rusqlite::Connection::open_in_memory().unwrap(),
        tmp_dir.path().to_str().unwrap(),
        true,
        b',',
        None,
        None,
    );
    assert!(result.is_ok());
    let msg = result.unwrap();
    assert!(msg.contains("Imported 1 CSV file(s)"));
    assert!(!msg.contains("readme"));
}

#[test]
fn test_import_gbk_encoded_csv() {
    let tmp_dir = TempDir::new().unwrap();
    // GBK: id,name\n1,中文\n
    let gbk_bytes: Vec<u8> = vec![
        0x69, 0x64, 0x2c, 0x6e, 0x61, 0x6d, 0x65, 0x0a, 0x31, 0x2c, 0xd6, 0xd0, 0xce, 0xc4, 0x0a,
    ];
    let path = tmp_dir.path().join("gbk_data.csv");
    let mut file = fs::File::create(&path).unwrap();
    file.write_all(&gbk_bytes).unwrap();

    let result = sqlimportcsvdir_impl(
        &rusqlite::Connection::open_in_memory().unwrap(),
        tmp_dir.path().to_str().unwrap(),
        true,
        b',',
        None,
        None,
    );
    assert!(result.is_ok());
    let msg = result.unwrap();
    assert!(msg.contains("Imported 1 CSV file(s)"));
    assert!(msg.contains("gbk_data"));
}

#[test]
fn test_import_invalid_dir() {
    let result = sqlimportcsvdir_impl(
        &rusqlite::Connection::open_in_memory().unwrap(),
        "/definitely/not/a/real/dir",
        true,
        b',',
        None,
        None,
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Read directory failed"));
}

#[test]
fn test_import_with_file_db_persistence() {
    let tmp_dir = TempDir::new().unwrap();
    let db_path = tmp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    // Create CSV files
    let csv_dir = tmp_dir.path().join("csv_files");
    fs::create_dir(&csv_dir).unwrap();
    create_test_csv(&csv_dir, "persistent.csv", "id,val\n1,hello\n");

    // Import
    let result = sqlimportcsvdir_impl(
        &rusqlite::Connection::open(&db_path).unwrap(),
        csv_dir.to_str().unwrap(),
        true,
        b',',
        None,
        None,
    );
    assert!(result.is_ok());

    // Re-open DB and verify
    let val = with_conn(db_path_str, |conn| {
        let mut stmt = conn.prepare("SELECT val FROM persistent").unwrap();
        let val: String = stmt.query_row([], |r| r.get(0)).unwrap();
        Ok(val)
    });
    assert_eq!(val.unwrap(), "hello");
}

#[test]
fn test_sanitize_table_name() {
    // This tests the internal sanitize logic indirectly through import
    let tmp_dir = TempDir::new().unwrap();
    create_test_csv(tmp_dir.path(), "2024-sales.csv", "a\n1\n");
    create_test_csv(tmp_dir.path(), "my data.csv", "b\n2\n");

    let result = sqlimportcsvdir_impl(
        &rusqlite::Connection::open_in_memory().unwrap(),
        tmp_dir.path().to_str().unwrap(),
        true,
        b',',
        None,
        None,
    );
    assert!(result.is_ok());
    let msg = result.unwrap();
    assert!(msg.contains("t_2024_sales") || msg.contains("2024_sales"));
    assert!(msg.contains("my_data"));
}
