use xll_rs::types::*;
use rusqlite::types::Value;
use crate::conn::with_conn;
use crate::xloper::sqlite_value_to_xloper;

pub fn sqlquery_impl(conn_str: &str, sql: &str) -> Result<XLOPER12, String> {
    with_conn(conn_str, |conn| {
        let mut stmt = conn.prepare(sql)
            .map_err(|e| format!("Prepare failed: {}", e))?;

        let col_count = stmt.column_count();
        let col_names: Vec<String> = stmt.column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();

        let mut rows_data: Vec<Vec<Value>> = Vec::new();
        let row_iter = stmt.query_map([], |row| {
            let mut values = Vec::with_capacity(col_count);
            for i in 0..col_count {
                values.push(row.get::<_, Value>(i)?);
            }
            Ok(values)
        }).map_err(|e| format!("Query failed: {}", e))?;

        for row in row_iter {
            rows_data.push(row.map_err(|e| format!("Row error: {}", e))?);
        }

        build_result_array(&col_names, &rows_data, col_count)
    })
}

pub fn sqlqueryp_impl(conn_str: &str, sql: &str, params: &[Value]) -> Result<XLOPER12, String> {
    with_conn(conn_str, |conn| {
        let mut stmt = conn.prepare(sql)
            .map_err(|e| format!("Prepare failed: {}", e))?;

        let col_count = stmt.column_count();
        let col_names: Vec<String> = stmt.column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();

        let mut rows_data: Vec<Vec<Value>> = Vec::new();
        let row_iter = stmt.query_map(rusqlite::params_from_iter(params), |row| {
            let mut values = Vec::with_capacity(col_count);
            for i in 0..col_count {
                values.push(row.get::<_, Value>(i)?);
            }
            Ok(values)
        }).map_err(|e| format!("Query failed: {}", e))?;

        for row in row_iter {
            rows_data.push(row.map_err(|e| format!("Row error: {}", e))?);
        }

        build_result_array(&col_names, &rows_data, col_count)
    })
}

fn append_pagination(sql: &str, limit: Option<i64>, offset: Option<i64>) -> String {
    let mut final_sql = sql.to_string();
    if !sql.to_uppercase().contains("LIMIT") {
        if let Some(l) = limit {
            final_sql.push_str(&format!(" LIMIT {}", l));
        }
        if let Some(o) = offset {
            final_sql.push_str(&format!(" OFFSET {}", o));
        }
    }
    final_sql
}

pub fn sqlqueryl_impl(conn_str: &str, sql: &str, limit: Option<i64>, offset: Option<i64>) -> Result<XLOPER12, String> {
    let final_sql = append_pagination(sql, limit, offset);
    sqlquery_impl(conn_str, &final_sql)
}

pub fn sqlqueryscalar_impl(conn_str: &str, sql: &str) -> Result<XLOPER12, String> {
    with_conn(conn_str, |conn| {
        let mut stmt = conn.prepare(sql)
            .map_err(|e| format!("Prepare failed: {}", e))?;

        let value: rusqlite::types::Value = stmt.query_row([], |row| {
            row.get::<_, rusqlite::types::Value>(0)
        }).map_err(|e| format!("Query failed: {}", e))?;

        Ok(sqlite_value_to_xloper(&value))
    })
}

pub fn build_result_array(col_names: &[String], rows_data: &[Vec<Value>], col_count: usize) -> Result<XLOPER12, String> {
    if col_count == 0 {
        return Ok(XLOPER12::from_str(""));
    }

    let total_rows = rows_data.len() + 1;
    let mut cells: Vec<XLOPER12> = Vec::with_capacity(total_rows * col_count);

    for name in col_names {
        cells.push(XLOPER12::from_str(name));
    }

    for row in rows_data {
        for value in row {
            cells.push(sqlite_value_to_xloper(value));
        }
    }

    let lparray = cells.as_mut_ptr();
    let rows = total_rows as i32;
    let columns = col_count as i32;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_pagination_basic() {
        let sql = "SELECT * FROM users";
        let result = append_pagination(sql, Some(10), Some(5));
        assert_eq!(result, "SELECT * FROM users LIMIT 10 OFFSET 5");
    }

    #[test]
    fn test_append_pagination_limit_only() {
        let sql = "SELECT * FROM users";
        let result = append_pagination(sql, Some(100), None);
        assert_eq!(result, "SELECT * FROM users LIMIT 100");
    }

    #[test]
    fn test_append_pagination_offset_only() {
        let sql = "SELECT * FROM users";
        let result = append_pagination(sql, None, Some(20));
        assert_eq!(result, "SELECT * FROM users OFFSET 20");
    }

    #[test]
    fn test_append_pagination_no_pagination() {
        let sql = "SELECT * FROM users";
        let result = append_pagination(sql, None, None);
        assert_eq!(result, "SELECT * FROM users");
    }

    #[test]
    fn test_append_pagination_already_has_limit() {
        let sql = "SELECT * FROM users LIMIT 50";
        let result = append_pagination(sql, Some(10), Some(5));
        assert_eq!(result, "SELECT * FROM users LIMIT 50");
    }

    #[test]
    fn test_append_pagination_case_insensitive_limit() {
        let sql = "SELECT * FROM users limit 50";
        let result = append_pagination(sql, Some(10), None);
        assert_eq!(result, "SELECT * FROM users limit 50");
    }

    #[test]
    fn test_append_pagination_subquery_with_limit() {
        let sql = "SELECT * FROM (SELECT * FROM t LIMIT 1) AS sub";
        let result = append_pagination(sql, Some(10), None);
        assert_eq!(result, "SELECT * FROM (SELECT * FROM t LIMIT 1) AS sub");
    }
}
