#![allow(clippy::not_unsafe_ptr_arg_deref)]
use rusqlite::Connection;
use crate::utils::types::{infer_column_type, make_valid_columns, normalize_sql_type};

pub fn sqlcreatetable_impl(
    conn: &Connection,
    table_name: &str,
    data_grid: Vec<Vec<String>>,
    explicit_columns: Option<Vec<String>>,
    explicit_types: Option<Vec<String>>,
) -> Result<String, String> {
    if data_grid.is_empty() {
        return Err("Data array is empty".to_string());
    }

    let data_col_count = data_grid.iter().map(|r| r.len()).max().unwrap_or(0);
    if data_col_count == 0 {
        return Err("Data array has no columns".to_string());
    }

    // let has_explicit_columns = explicit_columns.is_some();

    let (col_names, data_rows) = match explicit_columns {
        Some(cols) => {
            let cols = make_valid_columns(cols, data_col_count);
            if cols.len() != data_col_count {
                return Err(format!(
                    "Column count ({}) does not match data column count ({})",
                    cols.len(), data_col_count
                ));
            }
            (cols, data_grid)
        }
        None => {
            let first_row = data_grid[0].clone();
            let cols = make_valid_columns(first_row, data_col_count);
            let remaining = if data_grid.len() > 1 {
                data_grid[1..].to_vec()
            } else {
                Vec::new()
            };
            (cols, remaining)
        }
    };

    let col_count = col_names.len();

    let col_types = match explicit_types {
        Some(t) => {
            let mut types = t;
            while types.len() < col_count {
                types.push("TEXT".to_string());
            }
            types.truncate(col_count);
            types
        }
        None => {
            let infer_source: Vec<Vec<String>> = /*if has_explicit_columns && !data_rows.is_empty() {
                data_rows.clone()
            } else {
                data_rows.clone()
            };*/
            data_rows.clone();
            (0..col_count)
                .map(|col_idx| {
                    let col_values: Vec<String> = infer_source.iter()
                        .map(|row| row.get(col_idx).cloned().unwrap_or_default())
                        .collect();
                    infer_column_type(&col_values)
                })
                .collect()
        }
    };

    let safe_table = table_name.replace('"', "").trim().to_string();
    if safe_table.is_empty() {
        return Err("Table name cannot be empty".to_string());
    }

    let _ = conn.execute(
        &format!(r#"DROP TABLE IF EXISTS "{}""#, safe_table),
        []
    );

    let mut create_sql = format!(r#"CREATE TABLE "{}" ("#, safe_table);
    let cols_def: Vec<String> = col_names.iter().zip(col_types.iter())
        .map(|(name, ty)| {
            let safe_name = name.replace('"', "");
            let valid_type = normalize_sql_type(ty);
            format!(r#""{}" {}"#, safe_name, valid_type)
        })
        .collect();
    create_sql.push_str(&cols_def.join(", "));
    create_sql.push(')');

    conn.execute(&create_sql, [])
        .map_err(|e| format!("Create table failed: {}", e))?;

    let row_count = data_rows.len();
    if row_count > 0 {
        let placeholders = (0..col_count).map(|_| "?").collect::<Vec<_>>().join(", ");
        let insert_sql = format!(
            r#"INSERT INTO "{}" ({}) VALUES ({})"#,
            safe_table,
            col_names.iter().map(|c| format!(r#""{}""#, c.replace('"', ""))).collect::<Vec<_>>().join(", "),
            placeholders
        );

        let mut stmt = conn.prepare(&insert_sql)
            .map_err(|e| format!("Prepare insert failed: {}", e))?;

        let tx = conn.unchecked_transaction()
            .map_err(|e| format!("Begin transaction failed: {}", e))?;

        for row in &data_rows {
            let mut padded = row.clone();
            padded.resize(col_count, String::new());
            stmt.execute(rusqlite::params_from_iter(padded.iter()))
                .map_err(|e| format!("Insert failed: {}", e))?;
        }

        tx.commit().map_err(|e| format!("Commit failed: {}", e))?;
    }

    Ok(format!(
        "Table '{}' created: {} columns, {} rows",
        safe_table, col_count, row_count
    ))
}

pub fn sqlappendtable_impl(
    conn: &Connection,
    table_name: &str,
    data_grid: Vec<Vec<String>>,
) -> Result<String, String> {
    if data_grid.is_empty() {
        return Err("Data array is empty".to_string());
    }

    let data_col_count = data_grid.iter().map(|r| r.len()).max().unwrap_or(0);
    if data_col_count == 0 {
        return Err("Data array has no columns".to_string());
    }

    let safe_table = table_name.replace('"', "").trim().to_string();
    if safe_table.is_empty() {
        return Err("Table name cannot be empty".to_string());
    }

    // 检查表是否存在
    let table_exists: bool = conn.query_row(
        "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?",
        [&safe_table],
        |_| Ok(true),
    ).unwrap_or(false);

    if !table_exists {
        return Err(format!("Table '{}' does not exist. Use SqlCreateTable first.", safe_table));
    }

    // 获取表的列数
    let mut stmt = conn.prepare(&format!("PRAGMA table_info(\"{}\")", safe_table))
        .map_err(|e| format!("Prepare failed: {}", e))?;
    let col_count: usize = stmt.query_map([], |_| Ok(()))
        .map_err(|e| format!("Query failed: {}", e))?
        .count();

    if data_col_count != col_count {
        return Err(format!(
            "Data column count ({}) does not match table column count ({})",
            data_col_count, col_count
        ));
    }

    let row_count = data_grid.len();
    if row_count > 0 {
        let placeholders = (0..col_count).map(|_| "?").collect::<Vec<_>>().join(", ");
        let insert_sql = format!(
            r#"INSERT INTO "{}" VALUES ({})"#,
            safe_table,
            placeholders
        );

        let mut stmt = conn.prepare(&insert_sql)
            .map_err(|e| format!("Prepare insert failed: {}", e))?;

        let tx = conn.unchecked_transaction()
            .map_err(|e| format!("Begin transaction failed: {}", e))?;

        for row in &data_grid {
            let mut padded = row.clone();
            padded.resize(col_count, String::new());
            stmt.execute(rusqlite::params_from_iter(padded.iter()))
                .map_err(|e| format!("Insert failed: {}", e))?;
        }

        tx.commit().map_err(|e| format!("Commit failed: {}", e))?;
    }

    Ok(format!(
        "Table '{}': {} rows appended",
        safe_table, row_count
    ))
}
