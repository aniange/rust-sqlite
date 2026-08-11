use rusqlite::Connection;

pub fn sqlexec_impl(conn_str: &str, sql: &str) -> Result<usize, String> {
    let conn = Connection::open(conn_str).map_err(|e| e.to_string())?;
    let affected = conn.execute(sql, []).map_err(|e| e.to_string())?;
    Ok(affected)
}

pub fn sqlcreatedb_impl(path: &str) -> Result<String, String> {
    Connection::open(path)
        .map_err(|e| format!("Create DB failed: {}", e))?;
    Ok("Database created".to_string())
}
