#![allow(clippy::not_unsafe_ptr_arg_deref)]
use rusqlite::Connection;
use csv::StringRecord;
use encoding_rs::{UTF_8, GB18030};
use crate::utils::types::{infer_column_type, make_valid_columns, normalize_sql_type};

pub fn sqlimportcsv_impl(
    conn: &Connection,
    csv_path: &str,
    table_name: &str,
    has_header: bool,
    delimiter: u8,
    explicit_columns: Option<Vec<String>>,
    explicit_types: Option<Vec<String>>,
) -> Result<String, String> {
    let raw = std::fs::read(csv_path)
        .map_err(|e| format!("Read CSV failed: {}", e))?;

    let (cow, _, had_errors) = UTF_8.decode(&raw);
    let text = if had_errors {
        let (cow, _, _) = GB18030.decode(&raw);
        cow.into_owned()
    } else {
        cow.into_owned()
    };

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(delimiter)
        .flexible(true)
        .from_reader(text.as_bytes());

    let mut all_rows: Vec<StringRecord> = Vec::new();
    for result in rdr.records() {
        let record = result.map_err(|e| format!("CSV parse error: {}", e))?;
        all_rows.push(record);
    }

    if all_rows.is_empty() {
        return Err("CSV file is empty".to_string());
    }

    let col_count = all_rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if col_count == 0 {
        return Err("CSV has no columns".to_string());
    }

    let (col_names, data_rows) = if let Some(cols) = explicit_columns {
        let cols = make_valid_columns(cols, col_count);
        (cols, all_rows)
    } else if has_header {
        let first = all_rows.remove(0);
        let cols: Vec<String> = (0..col_count)
            .map(|i| first.get(i).unwrap_or("").trim().to_string())
            .collect();
        let cols = make_valid_columns(cols, col_count);
        (cols, all_rows)
    } else {
        let cols: Vec<String> = (0..col_count)
            .map(|i| format!("col_{}", i + 1))
            .collect();
        (cols, all_rows)
    };

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
            let sample: Vec<&StringRecord> = data_rows.iter().take(1000).collect();
            (0..col_count)
                .map(|col_idx| {
                    let values: Vec<String> = sample.iter()
                        .map(|row| row.get(col_idx).unwrap_or("").to_string())
                        .collect();
                    infer_column_type(&values)
                })
                .collect()
        }
    };

    let safe_table = table_name.replace('"', "").trim().to_string();
    if safe_table.is_empty() {
        return Err("Table name cannot be empty".to_string());
    }

    let _ = conn.execute(&format!(r#"DROP TABLE IF EXISTS "{}""#, safe_table), []);

    let cols_def: Vec<String> = col_names.iter().zip(col_types.iter())
        .map(|(name, ty)| {
            let safe_name = name.replace('"', "");
            let valid_type = normalize_sql_type(ty);
            format!(r#""{}" {}"#, safe_name, valid_type)
        })
        .collect();

    let create_sql = format!(
        r#"CREATE TABLE "{}" ({})"#,
        safe_table,
        cols_def.join(", ")
    );
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
            let mut values: Vec<String> = Vec::with_capacity(col_count);
            for i in 0..col_count {
                values.push(row.get(i).unwrap_or("").to_string());
            }
            stmt.execute(rusqlite::params_from_iter(values.iter()))
                .map_err(|e| format!("Insert failed: {}", e))?;
        }

        tx.commit().map_err(|e| format!("Commit failed: {}", e))?;
    }

    Ok(format!(
        "Table '{}' created from CSV: {} columns, {} rows",
        safe_table, col_count, row_count
    ))
}
