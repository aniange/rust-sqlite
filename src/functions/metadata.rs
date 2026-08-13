#![allow(clippy::not_unsafe_ptr_arg_deref)]
use crate::conn::with_conn;
use crate::functions::query::build_result_array;
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

pub fn sqlschema_impl(conn_str: &str, table_name: &str) -> Result<XLOPER12, String> {
    with_conn(conn_str, |conn| {
        let safe_table = table_name.replace('"', "");
        let sql = format!("PRAGMA table_info(\"{}\")", safe_table);
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| format!("Prepare failed: {}", e))?;

        let col_count = stmt.column_count();
        let col_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();

        let mut rows_data: Vec<Vec<rusqlite::types::Value>> = Vec::new();
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

pub fn sqlpragma_impl(conn_str: &str, pragma_name: &str) -> Result<XLOPER12, String> {
    with_conn(conn_str, |conn| {
        let safe_pragma = pragma_name.replace([';', '"'], "");
        let sql = format!("PRAGMA {}", safe_pragma);
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| format!("Prepare failed: {}", e))?;

        let col_count = stmt.column_count();
        let col_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();

        let mut rows_data: Vec<Vec<rusqlite::types::Value>> = Vec::new();
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
