use rust_sqlite::functions::exec::sqlexec_impl;
use rust_sqlite::functions::metadata::{escape_sqlite_id, sqlschema_impl};
use xll_rs::types::*;

fn temp_db_path(prefix: &str) -> String {
    let mut path = std::env::temp_dir();
    path.push(format!("{}_{}.db", prefix, std::process::id()));
    path.to_string_lossy().to_string()
}

/// 将 XLOPER12 多维数组解析为字符串网格，方便断言
unsafe fn xloper_to_string_grid(op: &XLOPER12) -> Vec<Vec<String>> {
    if op.base_type() != XLTYPE_MULTI {
        return vec![vec![op.as_string().unwrap_or_default()]];
    }
    let arr = &op.val.array;
    let rows = arr.rows as usize;
    let cols = arr.columns as usize;
    let mut result = Vec::with_capacity(rows);
    for r in 0..rows {
        let mut row = Vec::with_capacity(cols);
        for c in 0..cols {
            let cell = &*arr.lparray.add(r * cols + c);
            let s = match cell.base_type() {
                XLTYPE_STR => cell.as_string().unwrap_or_default(),
                XLTYPE_NUM => {
                    let n = cell.as_f64().unwrap_or(0.0);
                    if n.fract() == 0.0 {
                        format!("{:.0}", n)
                    } else {
                        n.to_string()
                    }
                }
                XLTYPE_INT => format!("{:.0}", cell.as_f64().unwrap_or(0.0)),
                XLTYPE_BOOL => cell.as_bool().unwrap_or(false).to_string(),
                _ => String::new(),
            };
            row.push(s);
        }
        result.push(row);
    }
    result
}

fn find_section(grid: &[Vec<String>], title: &str) -> Option<usize> {
    grid.iter()
        .position(|row| row.first().map(|s| s.as_str()) == Some(title))
}

// ========== escape_sqlite_id 纯函数测试 ==========

#[test]
fn test_escape_normal() {
    assert_eq!(escape_sqlite_id("users"), "\"users\"");
}

#[test]
fn test_escape_with_quote() {
    assert_eq!(escape_sqlite_id("user\"name"), "\"user\"\"name\"");
}

#[test]
fn test_escape_empty() {
    assert_eq!(escape_sqlite_id(""), "\"\"");
}

// ========== Schema 集成测试 ==========

#[test]
fn test_sqlschema_complete_table() {
    let db_path = temp_db_path("test_schema_complete");
    let _ = std::fs::remove_file(&db_path);

    sqlexec_impl(
        &db_path,
        "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL)",
    )
    .unwrap();
    sqlexec_impl(
        &db_path,
        "CREATE TABLE orders (
            id INTEGER PRIMARY KEY,
            user_id INTEGER NOT NULL,
            amount REAL DEFAULT 0.0,
            FOREIGN KEY (user_id) REFERENCES users(id)
        )",
    )
    .unwrap();
    sqlexec_impl(&db_path, "CREATE INDEX idx_orders_user ON orders(user_id)").unwrap();
    sqlexec_impl(
        &db_path,
        "CREATE UNIQUE INDEX idx_orders_amount ON orders(amount)",
    )
    .unwrap();

    let result = sqlschema_impl(&db_path, "orders");
    assert!(result.is_ok());

    let grid = unsafe { xloper_to_string_grid(&result.unwrap()) };

    // TABLE INFO
    let ti = find_section(&grid, "=== TABLE INFO ===").unwrap();
    assert!(grid[ti + 1].contains(&"orders".to_string()));
    assert!(grid[ti + 1]
        .iter()
        .any(|s| s.contains("CREATE TABLE orders")));

    // COLUMNS
    assert!(find_section(&grid, "=== COLUMNS ===").is_some());

    // INDEXES: 2 个索引
    let ii = find_section(&grid, "=== INDEXES ===").unwrap();
    let idx_count = grid[ii + 1..]
        .iter()
        .take_while(|r| !r.iter().all(|s| s.is_empty()))
        .filter(|r| r.get(0).map(|s| s.is_empty()).unwrap_or(false))
        .count();
    assert_eq!(idx_count, 2);

    // FOREIGN KEYS
    assert!(find_section(&grid, "=== FOREIGN KEYS ===").is_some());

    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn test_sqlschema_no_index_no_fk() {
    let db_path = temp_db_path("test_schema_simple");
    let _ = std::fs::remove_file(&db_path);

    sqlexec_impl(&db_path, "CREATE TABLE simple (id INTEGER, val TEXT)").unwrap();

    let result = sqlschema_impl(&db_path, "simple");
    assert!(result.is_ok());

    let grid = unsafe { xloper_to_string_grid(&result.unwrap()) };

    assert!(find_section(&grid, "=== COLUMNS ===").is_some());
    assert!(find_section(&grid, "=== INDEXES ===").is_none());
    assert!(find_section(&grid, "=== FOREIGN KEYS ===").is_none());

    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn test_sqlschema_special_table_name() {
    let db_path = temp_db_path("test_schema_special");
    let _ = std::fs::remove_file(&db_path);

    sqlexec_impl(&db_path, "CREATE TABLE \"weird\"\"table\" (id INTEGER)").unwrap();

    let result = sqlschema_impl(&db_path, "weird\"table");
    assert!(result.is_ok());

    let grid = unsafe { xloper_to_string_grid(&result.unwrap()) };
    assert!(find_section(&grid, "=== COLUMNS ===").is_some());

    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn test_sqlschema_table_not_exist() {
    let db_path = temp_db_path("test_schema_missing");
    let _ = std::fs::remove_file(&db_path);

    sqlexec_impl(&db_path, "CREATE TABLE dummy (id INTEGER)").unwrap();

    let result = sqlschema_impl(&db_path, "nonexistent");
    assert!(result.is_ok());

    let grid = unsafe { xloper_to_string_grid(&result.unwrap()) };

    assert!(find_section(&grid, "=== TABLE INFO ===").is_some());
    let ti = find_section(&grid, "=== TABLE INFO ===").unwrap();
    assert!(grid[ti + 1].iter().any(|s| s == "N/A"));
    assert!(find_section(&grid, "=== COLUMNS ===").is_none());

    let _ = std::fs::remove_file(&db_path);
}
