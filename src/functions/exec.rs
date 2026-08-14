#![allow(clippy::not_unsafe_ptr_arg_deref)]
use crate::conn::with_conn;

use encoding_rs::{GB18030, UTF_8};
use std::fs;
use std::path::Path;

pub fn sqlexec_impl(conn_str: &str, sql: &str) -> Result<usize, String> {
    with_conn(conn_str, |conn| {
        let affected = conn.execute(sql, []).map_err(|e| e.to_string())?;
        Ok(affected)
    })
}

// pub fn sqlexec_impl(conn_str: &str, sql: &str) -> Result<usize, String> {
//     with_conn(conn_str, |conn| {
//         let affected = conn.execute(sql, []).map_err(|e| e.to_string())?;
//         Ok(affected)
//     })
// }

pub fn sqlbegin_impl(conn_str: &str) -> Result<String, String> {
    with_conn(conn_str, |conn| {
        conn.execute("BEGIN", [])
            .map_err(|e| format!("BEGIN failed: {}", e))?;
        Ok("Transaction started".to_string())
    })
}

pub fn sqlcommit_impl(conn_str: &str) -> Result<String, String> {
    with_conn(conn_str, |conn| {
        conn.execute("COMMIT", [])
            .map_err(|e| format!("COMMIT failed: {}", e))?;
        Ok("Transaction committed".to_string())
    })
}

pub fn sqlrollback_impl(conn_str: &str) -> Result<String, String> {
    with_conn(conn_str, |conn| {
        conn.execute("ROLLBACK", [])
            .map_err(|e| format!("ROLLBACK failed: {}", e))?;
        Ok("Transaction rolled back".to_string())
    })
}

/// Execute a SQL script containing multiple statements.
///
/// If `script_or_path` points to an existing file, its content is read and executed.
/// Encoding is auto-detected: UTF-8 first, then GB18030 (compatible with GBK).
/// If the path does not exist, `script_or_path` is treated as raw SQL text.
pub fn sqlscript_impl(conn_str: &str, script_or_path: &str) -> Result<String, String> {
    let script = if Path::new(script_or_path).exists() {
        let raw = fs::read(script_or_path)
            .map_err(|e| format!("Failed to read script file '{}': {}", script_or_path, e))?;
        let (cow, _, had_errors) = UTF_8.decode(&raw);
        if had_errors {
            let (cow, _, _) = GB18030.decode(&raw);
            cow.into_owned()
        } else {
            cow.into_owned()
        }
    } else {
        script_or_path.to_string()
    };

    with_conn(conn_str, |conn| {
        conn.execute_batch(&script)
            .map_err(|e| format!("Script execution failed: {}", e))?;
        Ok("Script executed successfully".to_string())
    })
}
