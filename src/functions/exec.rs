use rusqlite::Connection;

pub fn sqlexec_impl(conn_str: &str, sql: &str) -> Result<usize, String> {
    let conn = Connection::open(conn_str).map_err(|e| e.to_string())?;
    let affected = conn.execute(sql, []).map_err(|e| e.to_string())?;
    Ok(affected)
}

pub fn sqlbegin_impl(conn_str: &str) -> Result<String, String> {
    let conn = Connection::open(conn_str).map_err(|e| e.to_string())?;
    conn.execute("BEGIN", [])
        .map_err(|e| format!("BEGIN failed: {}", e))?;
    Ok("Transaction started".to_string())
}

pub fn sqlcommit_impl(conn_str: &str) -> Result<String, String> {
    let conn = Connection::open(conn_str).map_err(|e| e.to_string())?;
    conn.execute("COMMIT", [])
        .map_err(|e| format!("COMMIT failed: {}", e))?;
    Ok("Transaction committed".to_string())
}

pub fn sqlrollback_impl(conn_str: &str) -> Result<String, String> {
    let conn = Connection::open(conn_str).map_err(|e| e.to_string())?;
    conn.execute("ROLLBACK", [])
        .map_err(|e| format!("ROLLBACK failed: {}", e))?;
    Ok("Transaction rolled back".to_string())
}
