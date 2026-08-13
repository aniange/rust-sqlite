/// Infer SQLite column type from sample values
pub fn infer_column_type(values: &[String]) -> String {
    let non_empty: Vec<&str> = values
        .iter()
        .filter(|s| !s.is_empty())
        .map(|s| s.as_str())
        .collect();

    if non_empty.is_empty() {
        return "TEXT".to_string();
    }

    let all_int = non_empty.iter().all(|s| s.parse::<i64>().is_ok());
    if all_int {
        return "INTEGER".to_string();
    }

    let all_real = non_empty.iter().all(|s| s.parse::<f64>().is_ok());
    if all_real {
        return "REAL".to_string();
    }

    "TEXT".to_string()
}

/// Sanitize column names: remove quotes, fill empty names, pad/truncate to expected count
pub fn make_valid_columns(names: Vec<String>, expected_count: usize) -> Vec<String> {
    let mut result: Vec<String> = names
        .into_iter()
        .enumerate()
        .map(|(i, name)| {
            let trimmed = name.replace('"', "").trim().to_string();
            if trimmed.is_empty() {
                format!("col_{}", i + 1)
            } else {
                trimmed
            }
        })
        .collect();

    while result.len() < expected_count {
        result.push(format!("col_{}", result.len() + 1));
    }
    result.truncate(expected_count);
    result
}

/// Normalize user-provided type aliases to SQLite canonical types
pub fn normalize_sql_type(ty: &str) -> &'static str {
    match ty.replace('"', "").trim().to_uppercase().as_str() {
        "INTEGER" | "INT" | "BIGINT" | "SMALLINT" | "TINYINT" => "INTEGER",
        "REAL" | "FLOAT" | "DOUBLE" | "DOUBLE PRECISION" => "REAL",
        "TEXT" | "VARCHAR" | "CHAR" | "STRING" | "NVARCHAR" | "CLOB" => "TEXT",
        "BLOB" | "BINARY" | "VARBINARY" => "BLOB",
        "NUMERIC" | "DECIMAL" | "NUMBER" => "NUMERIC",
        "BOOLEAN" | "BOOL" => "INTEGER",
        "DATE" | "DATETIME" | "TIMESTAMP" | "TIME" => "TEXT",
        _ => "TEXT",
    }
}

#[cfg(test)]
mod tests;
