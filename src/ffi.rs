#![allow(clippy::not_unsafe_ptr_arg_deref)]
use crate::conn::{
    clear_all, get_conn_cache, get_handle_counter, get_handle_map, resolve_conn, with_conn,
    MEMORY_DB_URI,
};
use crate::error::error_to_xloper;
use crate::functions::csv_export::sqlexportcsv_impl;
use crate::functions::csv_import::{sqlimportcsv_impl, sqlimportcsvdir_impl};
use crate::functions::exec::{
    sqlbegin_impl, sqlcommit_impl, sqlexec_impl, sqlrollback_impl, sqlscript_impl,
};
use crate::functions::metadata::{
    sqlencrypt_impl, sqlpragma_impl, sqlschema_impl, sqltables_impl, sqlversion_impl,
};
use crate::functions::query::{sqlquery_impl, sqlqueryl_impl, sqlqueryp_impl, sqlqueryscalar_impl};
use crate::functions::table::{sqlappendtable_impl, sqlcreatetable_impl};
use crate::xloper::{extract_conn_str, xloper_to_string_grid, xloper_to_string_list};
use std::sync::atomic::Ordering;
use xll_rs::register::{build_type_string, Reg};
use xll_rs::types::*;

#[no_mangle]
pub extern "system" fn sqlconnect(
    db_path: *mut XLOPER12,
    password: *mut XLOPER12,
) -> *mut XLOPER12 {
    let path = unsafe { extract_conn_str(db_path).unwrap_or_else(|| MEMORY_DB_URI.to_string()) };

    // 提取可选密码
    let password_opt = unsafe {
        let base = (*password).base_type();
        if base != XLTYPE_MISSING && base != XLTYPE_NIL {
            (*password).as_string()
        } else {
            None
        }
    };

    // 文件数据库：保存密码到映射表，供后续直接路径访问复用
    if path != MEMORY_DB_URI {
        if let Some(ref pwd) = password_opt {
            crate::conn::set_password(&path, pwd);
        }
    }

    // 通过 with_conn 创建/复用连接（内部自动处理 SQLCipher 密钥）
    if let Err(e) = crate::conn::with_conn(&path, |_| Ok(())) {
        return Box::into_raw(Box::new(error_to_xloper(&e)));
    }

    let handle = if path == MEMORY_DB_URI {
        "conn_memory".to_string()
    } else {
        let id = get_handle_counter().fetch_add(1, Ordering::SeqCst);
        format!("conn_{}", id)
    };
    get_handle_map()
        .lock()
        .unwrap()
        .insert(handle.clone(), path);

    Box::into_raw(Box::new(XLOPER12::from_str(&handle)))
}

#[no_mangle]
pub extern "system" fn sqldisconnect(handle: *mut XLOPER12) -> *mut XLOPER12 {
    let key = unsafe {
        match (*handle).as_string() {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };

    let mut map = get_handle_map().lock().unwrap();
    let mut cache = get_conn_cache().lock().unwrap();

    let msg = if key == "conn_memory" {
        map.remove(&key);
        "Memory connection handle released (data remains until Excel closes)".to_string()
    } else if let Some(path) = map.remove(&key) {
        cache.remove(&path);
        format!("Disconnected: {}", key)
    } else if cache.remove(&key).is_some() {
        format!("Disconnected: {}", key)
    } else {
        return Box::into_raw(Box::new(error_to_xloper(&format!(
            "Unknown handle or path: {}",
            key
        ))));
    };

    Box::into_raw(Box::new(XLOPER12::from_str(&msg)))
}

#[no_mangle]
pub extern "system" fn sqlquery(conn_str: *mut XLOPER12, sql: *mut XLOPER12) -> *mut XLOPER12 {
    let conn_raw = unsafe {
        match extract_conn_str(conn_str) {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };
    let query = unsafe {
        match (*sql).as_string() {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };

    let conn = resolve_conn(&conn_raw);
    match sqlquery_impl(&conn, &query) {
        Ok(result) => Box::into_raw(Box::new(result)),
        Err(e) => Box::into_raw(Box::new(error_to_xloper(&e))),
    }
}

#[no_mangle]
pub extern "system" fn sqlqueryp(
    conn_str: *mut XLOPER12,
    sql: *mut XLOPER12,
    p1: *mut XLOPER12,
    p2: *mut XLOPER12,
    p3: *mut XLOPER12,
    p4: *mut XLOPER12,
    p5: *mut XLOPER12,
) -> *mut XLOPER12 {
    let conn_raw = unsafe {
        match extract_conn_str(conn_str) {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };
    let query = unsafe {
        match (*sql).as_string() {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };

    let mut params = Vec::new();
    unsafe {
        for ptr in [p1, p2, p3, p4, p5] {
            if (*ptr).base_type() != XLTYPE_MISSING {
                params.push(xloper_to_sqlite_value(&*ptr));
            }
        }
    }

    let conn = resolve_conn(&conn_raw);
    match sqlqueryp_impl(&conn, &query, &params) {
        Ok(result) => Box::into_raw(Box::new(result)),
        Err(e) => Box::into_raw(Box::new(error_to_xloper(&e))),
    }
}

#[no_mangle]
pub extern "system" fn sqlqueryl(
    conn_str: *mut XLOPER12,
    sql: *mut XLOPER12,
    limit: *mut XLOPER12,
    offset: *mut XLOPER12,
) -> *mut XLOPER12 {
    let conn_raw = unsafe {
        match extract_conn_str(conn_str) {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };
    let query = unsafe {
        match (*sql).as_string() {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };

    let limit_val = unsafe {
        if (*limit).base_type() != XLTYPE_MISSING {
            Some((*limit).as_f64().unwrap_or(0.0) as i64)
        } else {
            None
        }
    };
    let offset_val = unsafe {
        if (*offset).base_type() != XLTYPE_MISSING {
            Some((*offset).as_f64().unwrap_or(0.0) as i64)
        } else {
            None
        }
    };

    let conn = resolve_conn(&conn_raw);
    match sqlqueryl_impl(&conn, &query, limit_val, offset_val) {
        Ok(result) => Box::into_raw(Box::new(result)),
        Err(e) => Box::into_raw(Box::new(error_to_xloper(&e))),
    }
}

#[no_mangle]
pub extern "system" fn sqlexec(conn_str: *mut XLOPER12, sql: *mut XLOPER12) -> *mut XLOPER12 {
    let conn_raw = unsafe {
        match extract_conn_str(conn_str) {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };
    let stmt = unsafe {
        match (*sql).as_string() {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };

    let conn = resolve_conn(&conn_raw);
    match sqlexec_impl(&conn, &stmt) {
        Ok(n) => Box::into_raw(Box::new(XLOPER12::from_f64(n as f64))),
        Err(e) => Box::into_raw(Box::new(error_to_xloper(&e))),
    }
}

#[no_mangle]
pub extern "system" fn sqltables(conn_str: *mut XLOPER12) -> *mut XLOPER12 {
    let conn_raw = unsafe {
        match extract_conn_str(conn_str) {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };

    let conn = resolve_conn(&conn_raw);
    match sqltables_impl(&conn) {
        Ok(result) => Box::into_raw(Box::new(result)),
        Err(e) => Box::into_raw(Box::new(error_to_xloper(&e))),
    }
}

#[no_mangle]
pub extern "system" fn sqlversion(conn_str: *mut XLOPER12) -> *mut XLOPER12 {
    let conn_raw = unsafe {
        match extract_conn_str(conn_str) {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };

    let conn = resolve_conn(&conn_raw);
    match sqlversion_impl(&conn) {
        Ok(result) => Box::into_raw(Box::new(result)),
        Err(e) => Box::into_raw(Box::new(error_to_xloper(&e))),
    }
}

#[no_mangle]
pub extern "system" fn sqlschema(
    conn_str: *mut XLOPER12,
    table_name: *mut XLOPER12,
) -> *mut XLOPER12 {
    let conn_raw = unsafe {
        match extract_conn_str(conn_str) {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };
    let name = unsafe {
        match (*table_name).as_string() {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };

    let conn = resolve_conn(&conn_raw);
    match sqlschema_impl(&conn, &name) {
        Ok(result) => Box::into_raw(Box::new(result)),
        Err(e) => Box::into_raw(Box::new(error_to_xloper(&e))),
    }
}

#[no_mangle]
pub extern "system" fn sqlpragma(
    conn_str: *mut XLOPER12,
    pragma_name: *mut XLOPER12,
) -> *mut XLOPER12 {
    let conn_raw = unsafe {
        match extract_conn_str(conn_str) {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };
    let name = unsafe {
        match (*pragma_name).as_string() {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };

    let conn = resolve_conn(&conn_raw);
    match sqlpragma_impl(&conn, &name) {
        Ok(result) => Box::into_raw(Box::new(result)),
        Err(e) => Box::into_raw(Box::new(error_to_xloper(&e))),
    }
}

#[no_mangle]
pub extern "system" fn sqlencrypt(
    source_path: *mut XLOPER12,
    target_path: *mut XLOPER12,
    target_password: *mut XLOPER12,
    source_password: *mut XLOPER12,
) -> *mut XLOPER12 {
    let source = unsafe {
        match (*source_path).as_string() {
            Some(s) if !s.is_empty() => s,
            _ => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };
    let target = unsafe {
        match (*target_path).as_string() {
            Some(s) if !s.is_empty() => s,
            _ => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };
    let target_pwd = unsafe {
        let base = (*target_password).base_type();
        if base != XLTYPE_MISSING && base != XLTYPE_NIL {
            (*target_password).as_string()
        } else {
            None
        }
    };
    let source_pwd = unsafe {
        let base = (*source_password).base_type();
        if base != XLTYPE_MISSING && base != XLTYPE_NIL {
            (*source_password).as_string()
        } else {
            None
        }
    };

    match sqlencrypt_impl(
        &source,
        &target,
        target_pwd.as_deref(),
        source_pwd.as_deref(),
    ) {
        Ok(msg) => Box::into_raw(Box::new(XLOPER12::from_str(&msg))),
        Err(e) => Box::into_raw(Box::new(error_to_xloper(&e))),
    }
}

#[no_mangle]
pub extern "system" fn sqlqueryscalar(
    conn_str: *mut XLOPER12,
    sql: *mut XLOPER12,
) -> *mut XLOPER12 {
    let conn_raw = unsafe {
        match extract_conn_str(conn_str) {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };
    let query = unsafe {
        match (*sql).as_string() {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };

    let conn = resolve_conn(&conn_raw);
    match sqlqueryscalar_impl(&conn, &query) {
        Ok(result) => Box::into_raw(Box::new(result)),
        Err(e) => Box::into_raw(Box::new(error_to_xloper(&e))),
    }
}

#[no_mangle]
pub extern "system" fn sqlappendtable(
    db: *mut XLOPER12,
    name: *mut XLOPER12,
    data: *mut XLOPER12,
) -> *mut XLOPER12 {
    let db_raw = unsafe {
        match extract_conn_str(db) {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };
    let table_name = unsafe {
        match (*name).as_string() {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };

    let data_grid = unsafe {
        match xloper_to_string_grid(data) {
            Some(g) => g,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };

    let db_path = resolve_conn(&db_raw);

    match with_conn(&db_path, |conn| {
        sqlappendtable_impl(conn, &table_name, data_grid)
    }) {
        Ok(msg) => Box::into_raw(Box::new(XLOPER12::from_str(&msg))),
        Err(e) => Box::into_raw(Box::new(error_to_xloper(&e))),
    }
}

#[no_mangle]
pub extern "system" fn sqlexportcsv(
    conn_str: *mut XLOPER12,
    sql: *mut XLOPER12,
    csv_path: *mut XLOPER12,
    delimiter: *mut XLOPER12,
) -> *mut XLOPER12 {
    let conn_raw = unsafe {
        match extract_conn_str(conn_str) {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };
    let conn = resolve_conn(&conn_raw);

    let query = unsafe {
        match (*sql).as_string() {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };

    let path = unsafe {
        match (*csv_path).as_string() {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };

    let delim = unsafe {
        if (*delimiter).base_type() == XLTYPE_MISSING || (*delimiter).base_type() == XLTYPE_NIL {
            b','
        } else {
            match (*delimiter).as_string() {
                Some(s) if !s.is_empty() => s.as_bytes()[0],
                _ => b',',
            }
        }
    };

    match sqlexportcsv_impl(&conn, &query, &path, delim) {
        Ok(msg) => Box::into_raw(Box::new(XLOPER12::from_str(&msg))),
        Err(e) => Box::into_raw(Box::new(error_to_xloper(&e))),
    }
}

#[no_mangle]
pub extern "system" fn sqlbegin(conn_str: *mut XLOPER12) -> *mut XLOPER12 {
    let conn_raw = unsafe {
        match extract_conn_str(conn_str) {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };
    let conn = resolve_conn(&conn_raw);
    match sqlbegin_impl(&conn) {
        Ok(msg) => Box::into_raw(Box::new(XLOPER12::from_str(&msg))),
        Err(e) => Box::into_raw(Box::new(error_to_xloper(&e))),
    }
}

#[no_mangle]
pub extern "system" fn sqlcommit(conn_str: *mut XLOPER12) -> *mut XLOPER12 {
    let conn_raw = unsafe {
        match extract_conn_str(conn_str) {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };
    let conn = resolve_conn(&conn_raw);
    match sqlcommit_impl(&conn) {
        Ok(msg) => Box::into_raw(Box::new(XLOPER12::from_str(&msg))),
        Err(e) => Box::into_raw(Box::new(error_to_xloper(&e))),
    }
}

#[no_mangle]
pub extern "system" fn sqlrollback(conn_str: *mut XLOPER12) -> *mut XLOPER12 {
    let conn_raw = unsafe {
        match extract_conn_str(conn_str) {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };
    let conn = resolve_conn(&conn_raw);
    match sqlrollback_impl(&conn) {
        Ok(msg) => Box::into_raw(Box::new(XLOPER12::from_str(&msg))),
        Err(e) => Box::into_raw(Box::new(error_to_xloper(&e))),
    }
}

#[no_mangle]
pub extern "system" fn sqlscript(conn_str: *mut XLOPER12, script: *mut XLOPER12) -> *mut XLOPER12 {
    let conn_raw = unsafe {
        match extract_conn_str(conn_str) {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };
    let stmt = unsafe {
        match (*script).as_string() {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };

    let conn = resolve_conn(&conn_raw);
    match sqlscript_impl(&conn, &stmt) {
        Ok(msg) => Box::into_raw(Box::new(XLOPER12::from_str(&msg))),
        Err(e) => Box::into_raw(Box::new(error_to_xloper(&e))),
    }
}

#[no_mangle]
pub extern "system" fn sqlcreatetable(
    db: *mut XLOPER12,
    name: *mut XLOPER12,
    data: *mut XLOPER12,
    columns: *mut XLOPER12,
    types: *mut XLOPER12,
) -> *mut XLOPER12 {
    let db_raw = unsafe {
        match extract_conn_str(db) {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };
    let table_name = unsafe {
        match (*name).as_string() {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };

    let data_grid = unsafe {
        match xloper_to_string_grid(data) {
            Some(g) => g,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };

    let cols = unsafe {
        if (*columns).base_type() == XLTYPE_MISSING {
            None
        } else {
            xloper_to_string_list(columns)
        }
    };

    let types_opt = unsafe {
        if (*types).base_type() == XLTYPE_MISSING {
            None
        } else {
            xloper_to_string_list(types)
        }
    };

    let db_path = resolve_conn(&db_raw);

    match with_conn(&db_path, |conn| {
        sqlcreatetable_impl(conn, &table_name, data_grid, cols, types_opt)
    }) {
        Ok(msg) => Box::into_raw(Box::new(XLOPER12::from_str(&msg))),
        Err(e) => Box::into_raw(Box::new(error_to_xloper(&e))),
    }
}

#[no_mangle]
pub extern "system" fn sqlimportcsv(
    conn_str: *mut XLOPER12,
    csv_path: *mut XLOPER12,
    table_name: *mut XLOPER12,
    has_header: *mut XLOPER12,
    delimiter: *mut XLOPER12,
    columns: *mut XLOPER12,
    types: *mut XLOPER12,
) -> *mut XLOPER12 {
    let conn_raw = unsafe {
        match extract_conn_str(conn_str) {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };
    let conn = resolve_conn(&conn_raw);

    let csv = unsafe {
        match (*csv_path).as_string() {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };

    let tbl = unsafe {
        match (*table_name).as_string() {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };

    let header = unsafe {
        if (*has_header).base_type() == XLTYPE_MISSING || (*has_header).base_type() == XLTYPE_NIL {
            true
        } else {
            (*has_header).as_bool().unwrap_or(true)
        }
    };

    let delim = unsafe {
        if (*delimiter).base_type() == XLTYPE_MISSING || (*delimiter).base_type() == XLTYPE_NIL {
            b','
        } else {
            match (*delimiter).as_string() {
                Some(s) if !s.is_empty() => s.as_bytes()[0],
                _ => b',',
            }
        }
    };

    let cols = unsafe {
        if (*columns).base_type() == XLTYPE_MISSING {
            None
        } else {
            xloper_to_string_list(columns)
        }
    };

    let types_opt = unsafe {
        if (*types).base_type() == XLTYPE_MISSING {
            None
        } else {
            xloper_to_string_list(types)
        }
    };

    match with_conn(&conn, |conn| {
        sqlimportcsv_impl(conn, &csv, &tbl, header, delim, cols, types_opt)
    }) {
        Ok(msg) => Box::into_raw(Box::new(XLOPER12::from_str(&msg))),
        Err(e) => Box::into_raw(Box::new(error_to_xloper(&e))),
    }
}

#[no_mangle]
pub extern "system" fn sqlimportcsvdir(
    conn_str: *mut XLOPER12,
    dir_path: *mut XLOPER12,
    has_header: *mut XLOPER12,
    delimiter: *mut XLOPER12,
    columns: *mut XLOPER12,
    types: *mut XLOPER12,
) -> *mut XLOPER12 {
    let conn_raw = unsafe {
        match extract_conn_str(conn_str) {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };
    let conn = resolve_conn(&conn_raw);

    let dir = unsafe {
        match (*dir_path).as_string() {
            Some(s) => s,
            None => return Box::into_raw(Box::new(XLOPER12::from_err(XLERR_VALUE))),
        }
    };

    let header = unsafe {
        if (*has_header).base_type() == XLTYPE_MISSING || (*has_header).base_type() == XLTYPE_NIL {
            true
        } else {
            (*has_header).as_bool().unwrap_or(true)
        }
    };

    let delim = unsafe {
        if (*delimiter).base_type() == XLTYPE_MISSING || (*delimiter).base_type() == XLTYPE_NIL {
            b','
        } else {
            match (*delimiter).as_string() {
                Some(s) if !s.is_empty() => s.as_bytes()[0],
                _ => b',',
            }
        }
    };

    let cols = unsafe {
        if (*columns).base_type() == XLTYPE_MISSING {
            None
        } else {
            xloper_to_string_list(columns)
        }
    };

    let types_opt = unsafe {
        if (*types).base_type() == XLTYPE_MISSING {
            None
        } else {
            xloper_to_string_list(types)
        }
    };

    match with_conn(&conn, |conn| {
        sqlimportcsvdir_impl(conn, &dir, header, delim, cols, types_opt)
    }) {
        Ok(msg) => Box::into_raw(Box::new(XLOPER12::from_str(&msg))),
        Err(e) => Box::into_raw(Box::new(error_to_xloper(&e))),
    }
}

#[no_mangle]
pub extern "system" fn xlAutoOpen() -> i32 {
    let reg = Reg::new();

    let _ = reg.add(
    "sqlconnect",
    &build_type_string('Q', &['Q', 'Q'], false, false, false),
    "SqlConnect",
    "db_path, password",
    "SQLite",
    "Connect to a SQLite database and return a handle for reuse. Omit path to use in-memory database. Provide password to open SQLCipher encrypted databases.",
    &[
        "Full path to the SQLite database file, or omit to use shared memory database",
        "Optional: password for SQLCipher encrypted database. Omit for unencrypted databases.",
    ],
);

    let _ = reg.add(
        "sqldisconnect",
        &build_type_string('Q', &['Q'], false, false, false),
        "SqlDisconnect",
        "handle_or_path",
        "SQLite",
        "Disconnect a database handle or close a cached connection",
        &["Connection handle (e.g. conn_1, conn_memory) or full database path"],
    );

    let _ = reg.add(
        "sqlquery",
        &build_type_string('Q', &['Q', 'Q'], false, false, false),
        "SqlQuery",
        "conn_str, sql",
        "SQLite",
        "Execute SELECT query and return results as a 2D array",
        &[
            "Database handle, full file path, or omit for in-memory database",
            "The SQL SELECT statement to execute",
        ],
    );

    let _ = reg.add(
        "sqlqueryp",
        &build_type_string(
            'Q',
            &['Q', 'Q', 'Q', 'Q', 'Q', 'Q', 'Q'],
            false,
            false,
            false,
        ),
        "SqlQueryP",
        "conn_str, sql, p1, p2, p3, p4, p5",
        "SQLite",
        "Execute query with bound parameters (up to 5), prevents SQL injection",
        &[
            "Database handle, full file path, or omit for in-memory database",
            "SQL statement using ? as placeholders",
            "First parameter value (optional)",
            "Second parameter value (optional)",
            "Third parameter value (optional)",
            "Fourth parameter value (optional)",
            "Fifth parameter value (optional)",
        ],
    );

    let _ = reg.add(
        "sqlqueryl",
        &build_type_string('Q', &['Q', 'Q', 'Q', 'Q'], false, false, false),
        "SqlQueryL",
        "conn_str, sql, limit, offset",
        "SQLite",
        "Execute query with LIMIT/OFFSET pagination",
        &[
            "Database handle, full file path, or omit for in-memory database",
            "The SQL SELECT statement (do not include LIMIT/OFFSET)",
            "Maximum number of rows to return (optional)",
            "Number of rows to skip before returning results (optional)",
        ],
    );

    let _ = reg.add(
        "sqlexec",
        &build_type_string('Q', &['Q', 'Q'], false, false, false),
        "SqlExec",
        "conn_str, sql",
        "SQLite",
        "Execute INSERT/UPDATE/DELETE/CREATE TABLE and return affected row count",
        &[
            "Database handle, full file path, or omit for in-memory database",
            "The SQL statement to execute",
        ],
    );

    let _ = reg.add(
        "sqltables",
        &build_type_string('Q', &['Q'], false, false, false),
        "SqlTables",
        "conn_str",
        "SQLite",
        "List all tables in the specified database",
        &["Database handle, full file path, or omit for in-memory database"],
    );

    let _ = reg.add(
        "sqlversion",
        &build_type_string('Q', &['Q'], false, false, false),
        "SqlVersion",
        "conn_str",
        "SQLite",
        "Return the SQLite engine version number",
        &["Database handle, full file path, or omit for in-memory database"],
    );

    let _ = reg.add(
        "sqlschema",
        &build_type_string('Q', &['Q', 'Q'], false, false, false),
        "SqlSchema",
        "conn_str, table_name",
        "SQLite",
        "Return complete schema of a table: columns, indexes, foreign keys and CREATE TABLE SQL",
        &[
            "Database handle, full file path, or omit for in-memory database",
            "Name of the table to inspect",
        ],
    );

    let _ = reg.add(
        "sqlpragma",
        &build_type_string('Q', &['Q', 'Q'], false, false, false),
        "SqlPragma",
        "conn_str, pragma_name",
        "SQLite",
        "Execute a PRAGMA statement and return results",
        &[
            "Database handle, full file path, or omit for in-memory database",
            "PRAGMA name and arguments, e.g. 'journal_mode' or 'table_info(users)'",
        ],
    );

    let _ = reg.add(
    "sqlencrypt",
    &build_type_string('Q', &['Q', 'Q', 'Q', 'Q'], false, false, false),
    "SqlEncrypt",
    "source_path, target_path, target_password, source_password",
    "SQLite",
    "Export source database to target with SQLCipher encryption or decryption. Target must not exist.",
    &[
        "Full path to source database file",
        "Full path for new target database file (must not exist)",
        "Target password: provide to encrypt, omit or empty to create plaintext",
        "Optional: source password if source is encrypted and not previously opened with SqlConnect",
    ],
);

    let _ = reg.add(
        "sqlqueryscalar",
        &build_type_string('Q', &['Q', 'Q'], false, false, false),
        "SqlQueryScalar",
        "conn_str, sql",
        "SQLite",
        "Execute query and return only the first row, first column as a scalar value",
        &[
            "Database handle, full file path, or omit for in-memory database",
            "The SQL SELECT statement (should return exactly one value)",
        ],
    );

    let _ = reg.add(
        "sqlappendtable",
        &build_type_string('Q', &['Q', 'Q', 'Q'], false, false, false),
        "SqlAppendTable",
        "db_path, table_name, data",
        "SQLite",
        "Append Excel data rows to an existing table. Column count must match.",
        &[
            "Database handle, full file path, or omit for in-memory database",
            "Name of the existing table to append to",
            "Excel data range (2D array) without headers",
        ],
    );

    let _ = reg.add(
        "sqlexportcsv",
        &build_type_string('Q', &['Q', 'Q', 'Q', 'Q', 'Q'], false, false, false),
        "SqlExportCsv",
        "conn_str, sql, csv_path, delimiter",
        "SQLite",
        "Execute query and export results to a CSV file",
        &[
            "Database handle, full file path, or omit for in-memory database",
            "The SQL SELECT statement to export",
            "Full path for the output CSV file",
            "Optional: delimiter character, default comma ','",
        ],
    );

    let _ = reg.add(
        "sqlbegin",
        &build_type_string('Q', &['Q'], false, false, false),
        "SqlBegin",
        "conn_str",
        "SQLite",
        "Start a database transaction",
        &["Database handle, full file path, or omit for in-memory database"],
    );

    let _ = reg.add(
        "sqlcommit",
        &build_type_string('Q', &['Q'], false, false, false),
        "SqlCommit",
        "conn_str",
        "SQLite",
        "Commit the current transaction",
        &["Database handle, full file path, or omit for in-memory database"],
    );

    let _ = reg.add(
        "sqlrollback",
        &build_type_string('Q', &['Q'], false, false, false),
        "SqlRollback",
        "conn_str",
        "SQLite",
        "Rollback the current transaction",
        &["Database handle, full file path, or omit for in-memory database"],
    );

    let _ = reg.add(
        "sqlscript",
        &build_type_string('Q', &['Q', 'Q'], false, false, false),
        "SqlScript",
        "conn_str, script_or_path",
        "SQLite",
        "Execute a SQL script (multiple statements) or run a .sql file. Auto-detects file path vs raw SQL and encoding (UTF-8/GBK).",
        &[
            "Database handle, full file path, or omit for in-memory database",
            "SQL script text, or full path to a .sql file (auto-detects UTF-8 / GBK encoding)",
        ],
    );

    let _ = reg.add(
        "sqlcreatetable",
        &build_type_string('Q', &['Q', 'Q', 'Q', 'Q', 'Q'], false, false, false),
        "SqlCreateTable",
        "db_path, table_name, data, columns, types",
        "SQLite",
        "Create table from Excel data range. Omit columns/types to use first row as headers with auto-detected types.",
        &[
            "Database handle, full file path, or omit for in-memory database",
            "Name for the new table",
            "Excel data range (2D array). First row used as headers if columns omitted",
            "Optional: column names. Can be a range (e.g. A1:D1) or array literal {\"id\",\"name\"}",
            "Optional: column types. Can be a range (e.g. F1:F4) or array literal {\"INTEGER\",\"TEXT\"}. If omitted, auto-detected from data",
        ],
    );

    let _ = reg.add(
        "sqlimportcsv",
        &build_type_string('Q', &['Q', 'Q', 'Q', 'Q', 'Q', 'Q', 'Q'], false, false, false),
        "SqlImportCsv",
        "conn_str, csv_path, table_name, has_header, delimiter, columns, types",
        "SQLite",
        "Import a CSV file into SQLite. Auto-detects headers and column types. Supports custom delimiters and encoding (UTF-8/GBK).",
        &[
            "Database handle, full file path, or omit for in-memory database",
            "Full path to the CSV file (e.g. C:\\data\\file.csv)",
            "Name for the new table",
            "Optional: does CSV have header row? TRUE/FALSE, default TRUE",
            "Optional: delimiter character, default comma ','",
            "Optional: column names. Can be a range or array literal",
            "Optional: column types. Can be a range or array literal",
        ],
    );

    let _ = reg.add(
        "sqlimportcsvdir",
        &build_type_string('Q', &['Q', 'Q', 'Q', 'Q', 'Q', 'Q'], false, false, false),
        "SqlImportCsvDir",
        "conn_str, dir_path, has_header, delimiter, columns, types",
        "SQLite",
        "Batch import all CSV files from a directory. Each file becomes a table named after the file. Auto-detects UTF-8/GBK encoding per file.",
        &[
            "Database handle, full file path, or omit for in-memory database",
            "Full path to the directory containing CSV files",
            "Optional: does CSV have header row? TRUE/FALSE, default TRUE",
            "Optional: delimiter character, default comma ','",
            "Optional: column names for ALL files. Can be a range or array literal",
            "Optional: column types for ALL files. Can be a range or array literal",
        ],
    );

    1
}

#[no_mangle]
pub extern "system" fn xlAutoClose() -> i32 {
    clear_all();
    1
}

fn xloper_to_sqlite_value(op: &XLOPER12) -> rusqlite::types::Value {
    match op.base_type() {
        XLTYPE_NUM => rusqlite::types::Value::Real(op.as_f64().unwrap_or(0.0)),
        XLTYPE_INT => rusqlite::types::Value::Integer(op.as_f64().unwrap_or(0.0) as i64),
        XLTYPE_BOOL => {
            rusqlite::types::Value::Integer(if op.as_bool().unwrap_or(false) { 1 } else { 0 })
        }
        XLTYPE_STR => rusqlite::types::Value::Text(op.as_string().unwrap_or_default()),
        _ => rusqlite::types::Value::Null,
    }
}
