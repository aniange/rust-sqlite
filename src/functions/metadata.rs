#![allow(clippy::not_unsafe_ptr_arg_deref)]
use crate::conn::with_conn;
use crate::functions::query::build_result_array;
use crate::xloper::sqlite_value_to_xloper;
use rusqlite::types::Value;
use xll_rs::types::*;

pub fn sqltables_impl(conn_str: &str) -> Result<XLOPER12, String> {
    with_conn(conn_str, |conn| {
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .map_err(|e| format!("Prepare failed: {}", e))?;
        let names: Vec<String> = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| format!("Query failed: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Row error: {}", e))?;

        let mut cells: Vec<XLOPER12> = names.iter().map(|n| XLOPER12::from_str(n)).collect();
        let lparray = cells.as_mut_ptr();
        let rows = names.len() as i32;
        let columns = 1i32;
        std::mem::forget(cells);

        Ok(XLOPER12 {
            val: xll_rs::types::XLOPER12Val {
                array: std::mem::ManuallyDrop::new(xll_rs::types::XLOPER12Array {
                    lparray,
                    rows,
                    columns,
                }),
            },
            xltype: XLTYPE_MULTI | XLBIT_DLL_FREE,
        })
    })
}

pub fn sqlversion_impl(conn_str: &str) -> Result<XLOPER12, String> {
    with_conn(conn_str, |conn| {
        let mut stmt = conn
            .prepare("SELECT sqlite_version() as version")
            .map_err(|e| format!("Prepare failed: {}", e))?;
        let version: String = stmt
            .query_row([], |r| r.get(0))
            .map_err(|e| format!("Query failed: {}", e))?;
        Ok(XLOPER12::from_str(&version))
    })
}

/// 安全转义 SQLite 标识符：双引号包裹，内部双引号转义为两个
pub fn escape_sqlite_id(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

pub fn sqlschema_impl(conn_str: &str, table_name: &str) -> Result<XLOPER12, String> {
    with_conn(conn_str, |conn| {
        let safe_table = escape_sqlite_id(table_name);
        const MAX_COLS: usize = 8;

        // ========== 1. 建表 SQL ==========
        let create_sql: String = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE name = ?1 AND type = 'table'",
                [table_name],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| "N/A".to_string());

        // 辅助：将行填充到统一列宽
        let pad_row = |row: &mut Vec<Value>| {
            while row.len() < MAX_COLS {
                row.push(Value::Text("".to_string()));
            }
        };

        let mut all_rows: Vec<Vec<Value>> = Vec::new();

        // ===== Section: TABLE INFO =====
        all_rows.push({
            let mut r = vec![Value::Text("=== TABLE INFO ===".to_string())];
            pad_row(&mut r);
            r
        });
        all_rows.push({
            let mut r = vec![
                Value::Text("name".to_string()),
                Value::Text(table_name.to_string()),
                Value::Text("sql".to_string()),
                Value::Text(create_sql),
            ];
            pad_row(&mut r);
            r
        });
        all_rows.push({
            let r = vec![Value::Text("".to_string()); MAX_COLS];
            r
        });

        // ===== Section: COLUMNS (PRAGMA table_info) =====
        let col_names: Vec<String>;
        let mut col_rows: Vec<Vec<Value>> = Vec::new();
        {
            let sql = format!("PRAGMA table_info({})", safe_table);
            let mut stmt = conn
                .prepare(&sql)
                .map_err(|e| format!("Prepare table_info failed: {}", e))?;
            col_names = stmt.column_names().iter().map(|s| s.to_string()).collect();
            let cc = col_names.len();
            let iter = stmt
                .query_map([], |row| {
                    let mut v = Vec::with_capacity(cc);
                    for i in 0..cc {
                        v.push(row.get::<_, Value>(i)?);
                    }
                    Ok(v)
                })
                .map_err(|e| format!("Query table_info failed: {}", e))?;
            for r in iter {
                col_rows.push(r.map_err(|e| format!("Row error: {}", e))?);
            }
        }

        if !col_rows.is_empty() {
            all_rows.push({
                let mut r = vec![Value::Text("=== COLUMNS ===".to_string())];
                for n in &col_names {
                    r.push(Value::Text(n.clone()));
                }
                pad_row(&mut r);
                r
            });
            for row in &col_rows {
                let mut r = vec![Value::Text("".to_string())];
                r.extend(row.clone());
                pad_row(&mut r);
                all_rows.push(r);
            }
            all_rows.push({
                let r = vec![Value::Text("".to_string()); MAX_COLS];
                r
            });
        }

        // ===== Section: INDEXES =====
        let mut index_rows: Vec<Vec<Value>> = Vec::new();
        {
            let sql = format!("PRAGMA index_list({})", safe_table);
            let mut stmt = conn
                .prepare(&sql)
                .map_err(|e| format!("Prepare index_list failed: {}", e))?;
            let cc = stmt.column_count();
            let iter = stmt
                .query_map([], |row| {
                    let mut v = Vec::with_capacity(cc);
                    for i in 0..cc {
                        v.push(row.get::<_, Value>(i)?);
                    }
                    Ok(v)
                })
                .map_err(|e| format!("Query index_list failed: {}", e))?;

            for r in iter {
                let row_values = r.map_err(|e| format!("Row error: {}", e))?;
                let idx_name: String = match row_values.get(1) {
                    Some(Value::Text(s)) => s.clone(),
                    _ => continue,
                };
                let safe_idx = escape_sqlite_id(&idx_name);
                let col_sql = format!("PRAGMA index_info({})", safe_idx);
                let mut col_stmt = conn
                    .prepare(&col_sql)
                    .map_err(|e| format!("Prepare index_info failed: {}", e))?;
                let col_iter = col_stmt
                    .query_map([], |r| r.get::<_, String>(2))
                    .map_err(|e| format!("Query index_info failed: {}", e))?;
                let mut cols = Vec::new();
                for c in col_iter {
                    cols.push(c.map_err(|e| format!("Row error: {}", e))?);
                }
                let mut extended = row_values.clone();
                extended.push(Value::Text(cols.join(", ")));
                index_rows.push(extended);
            }
        }

        if !index_rows.is_empty() {
            all_rows.push({
                let mut r = vec![
                    Value::Text("=== INDEXES ===".to_string()),
                    Value::Text("seq".to_string()),
                    Value::Text("name".to_string()),
                    Value::Text("unique".to_string()),
                    Value::Text("origin".to_string()),
                    Value::Text("partial".to_string()),
                    Value::Text("columns".to_string()),
                ];
                pad_row(&mut r);
                r
            });
            for row in &index_rows {
                let mut r = vec![Value::Text("".to_string())];
                r.extend(row.clone());
                pad_row(&mut r);
                all_rows.push(r);
            }
            all_rows.push({
                let r = vec![Value::Text("".to_string()); MAX_COLS];
                r
            });
        }

        // ===== Section: FOREIGN KEYS =====
        let fk_names: Vec<String>;
        let mut fk_rows: Vec<Vec<Value>> = Vec::new();
        {
            let sql = format!("PRAGMA foreign_key_list({})", safe_table);
            let mut stmt = conn
                .prepare(&sql)
                .map_err(|e| format!("Prepare foreign_key_list failed: {}", e))?;
            fk_names = stmt.column_names().iter().map(|s| s.to_string()).collect();
            let cc = fk_names.len();
            let iter = stmt
                .query_map([], |row| {
                    let mut v = Vec::with_capacity(cc);
                    for i in 0..cc {
                        v.push(row.get::<_, Value>(i)?);
                    }
                    Ok(v)
                })
                .map_err(|e| format!("Query foreign_key_list failed: {}", e))?;
            for r in iter {
                fk_rows.push(r.map_err(|e| format!("Row error: {}", e))?);
            }
        }

        if !fk_rows.is_empty() {
            all_rows.push({
                let mut r = vec![Value::Text("=== FOREIGN KEYS ===".to_string())];
                for n in &fk_names {
                    r.push(Value::Text(n.clone()));
                }
                pad_row(&mut r);
                r
            });
            for row in &fk_rows {
                let mut r = vec![Value::Text("".to_string())];
                r.extend(row.clone());
                pad_row(&mut r);
                all_rows.push(r);
            }
        }

        // 构建 XLOPER12 数组
        let total_cells = all_rows.len() * MAX_COLS;
        let mut cells: Vec<XLOPER12> = Vec::with_capacity(total_cells);
        for row in &all_rows {
            for value in row {
                cells.push(sqlite_value_to_xloper(value));
            }
        }

        let lparray = cells.as_mut_ptr();
        let rows = all_rows.len() as i32;
        let columns = MAX_COLS as i32;
        std::mem::forget(cells);

        Ok(XLOPER12 {
            val: xll_rs::types::XLOPER12Val {
                array: std::mem::ManuallyDrop::new(xll_rs::types::XLOPER12Array {
                    lparray,
                    rows,
                    columns,
                }),
            },
            xltype: XLTYPE_MULTI | XLBIT_DLL_FREE,
        })
    })
}

pub fn sqlpragma_impl(conn_str: &str, pragma_name: &str) -> Result<XLOPER12, String> {
    with_conn(conn_str, |conn| {
        let safe_pragma = pragma_name.replace([';', '"'], "");
        let sql = format!("PRAGMA {}", safe_pragma);
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| format!("Prepare failed: {}", e))?;

        let col_count = stmt.column_count();
        let col_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();

        let mut rows_data: Vec<Vec<Value>> = Vec::new();
        let row_iter = stmt
            .query_map([], |row| {
                let mut values = Vec::with_capacity(col_count);
                for i in 0..col_count {
                    values.push(row.get::<_, rusqlite::types::Value>(i)?);
                }
                Ok(values)
            })
            .map_err(|e| format!("Query failed: {}", e))?;

        for row in row_iter {
            rows_data.push(row.map_err(|e| format!("Row error: {}", e))?);
        }

        build_result_array(&col_names, &rows_data, col_count)
    })
}

pub fn sqlencrypt_impl(
    source_path: &str,
    target_path: &str,
    target_password: Option<&str>,
    source_password: Option<&str>,
) -> Result<String, String> {
    // 基础校验
    if source_path == target_path {
        return Err("Source and target cannot be the same file".to_string());
    }
    if std::path::Path::new(target_path).exists() {
        return Err(format!("Target file already exists: {}", target_path));
    }

    // 如果显式提供了源密码，临时注册到密码映射表（覆盖已有记录）
    if let Some(pwd) = source_password {
        crate::conn::set_password(source_path, pwd);
    }

    with_conn(source_path, |conn| {
        // 验证源库可读（确认密钥正确或文件为明文）
        conn.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))
            .map_err(|e| format!("Cannot read source database (wrong key?): {}", e))?;

        // 构造 ATTACH 语句
        let safe_target = target_path.replace('\'', "''");
        let key_clause = match target_password {
            Some(pwd) if !pwd.is_empty() => {
                let safe_pwd = pwd.replace('\'', "''");
                format!("KEY '{}'", safe_pwd)
            }
            _ => "KEY ''".to_string(), // 空密码 = 明文（解密导出）
        };

        // 1. ATTACH 目标库
        let attach_sql = format!(
            "ATTACH DATABASE '{}' AS encrypted_db {};",
            safe_target, key_clause
        );
        conn.execute_batch(&attach_sql)
            .map_err(|e| format!("ATTACH target failed: {}", e))?;

        // 2. 执行 sqlcipher_export
        let export_result =
            conn.query_row::<(), _, _>("SELECT sqlcipher_export('encrypted_db');", [], |_| Ok(()));

        if let Err(e) = export_result {
            // 清理：DETACH + 删除可能残留的不完整文件
            let _ = conn.execute_batch("DETACH DATABASE encrypted_db;");
            let _ = std::fs::remove_file(target_path);
            return Err(format!("Export failed: {}", e));
        }

        // 3. DETACH
        conn.execute_batch("DETACH DATABASE encrypted_db;")
            .map_err(|e| format!("DETACH failed: {}", e))?;

        // 4. 确认目标文件已生成
        if !std::path::Path::new(target_path).exists() {
            return Err("Target file was not created".to_string());
        }

        // 5. 统计源库表数量用于返回信息
        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let action = if target_password.map_or(false, |p| !p.is_empty()) {
            "Encrypted"
        } else {
            "Decrypted"
        };

        Ok(format!(
            "{} {} tables from '{}' to '{}'",
            action, count, source_path, target_path
        ))
    })
}
