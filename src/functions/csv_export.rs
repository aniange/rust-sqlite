#![allow(clippy::not_unsafe_ptr_arg_deref)]
use csv::WriterBuilder;
use crate::conn::with_conn;

pub fn sqlexportcsv_impl(
    conn_str: &str,
    sql: &str,
    csv_path: &str,
    delimiter: u8,
) -> Result<String, String> {
    with_conn(conn_str, |conn| {
        let mut stmt = conn.prepare(sql)
            .map_err(|e| format!("Prepare failed: {}", e))?;

        let col_count = stmt.column_count();
        let col_names: Vec<String> = stmt.column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();

        // 收集所有数据行
        let mut rows_data: Vec<Vec<rusqlite::types::Value>> = Vec::new();
        let row_iter = stmt.query_map([], |row| {
            let mut values = Vec::with_capacity(col_count);
            for i in 0..col_count {
                values.push(row.get::<_, rusqlite::types::Value>(i)?);
            }
            Ok(values)
        }).map_err(|e| format!("Query failed: {}", e))?;

        for row in row_iter {
            rows_data.push(row.map_err(|e| format!("Row error: {}", e))?);
        }

        // 创建 CSV writer
        let mut wtr = WriterBuilder::new()
            .delimiter(delimiter)
            .from_path(csv_path)
            .map_err(|e| format!("Create CSV writer failed: {}", e))?;

        // 写入列名
        wtr.write_record(&col_names)
            .map_err(|e| format!("Write header failed: {}", e))?;

        // 写入数据行
        for row in &rows_data {
            let record: Vec<String> = row.iter().map(|v| {
                match v {
                    rusqlite::types::Value::Null => String::new(),
                    rusqlite::types::Value::Integer(i) => i.to_string(),
                    rusqlite::types::Value::Real(f) => f.to_string(),
                    rusqlite::types::Value::Text(s) => s.clone(),
                    rusqlite::types::Value::Blob(b) => {
                        b.iter().map(|byte| format!("{:02X}", byte)).collect()
                    }
                }
            }).collect();
            wtr.write_record(&record)
                .map_err(|e| format!("Write record failed: {}", e))?;
        }

        wtr.flush().map_err(|e| format!("Flush failed: {}", e))?;

        Ok(format!(
            "Exported {} rows to '{}'",
            rows_data.len(), csv_path
        ))
    })
}
