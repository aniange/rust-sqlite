#![allow(clippy::not_unsafe_ptr_arg_deref)]
use rusqlite::{Connection, OpenFlags};
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::Mutex;

pub const MEMORY_DB_URI: &str = "file:sqlite_xll_shared?mode=memory&cache=shared";

lazy_static::lazy_static! {
    static ref CONN_CACHE: Mutex<HashMap<String, Connection>> = Mutex::new(HashMap::new());
    static ref HANDLE_MAP: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
    static ref HANDLE_COUNTER: AtomicUsize = AtomicUsize::new(1);
    static ref PASSWORD_MAP: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

/// 为指定数据库路径保存 SQLCipher 密码，供后续直接路径访问自动使用
pub fn set_password(path: &str, password: &str) {
    let mut map = PASSWORD_MAP.lock().unwrap();
    map.insert(path.to_string(), password.to_string());
}

pub fn with_conn<F, T>(path: &str, f: F) -> Result<T, String>
where
    F: FnOnce(&Connection) -> Result<T, String>,
{
    let mut cache = CONN_CACHE.lock().unwrap();
    if !cache.contains_key(path) {
        let conn = if path == MEMORY_DB_URI {
            Connection::open_with_flags(
                path,
                OpenFlags::SQLITE_OPEN_URI
                    | OpenFlags::SQLITE_OPEN_READ_WRITE
                    | OpenFlags::SQLITE_OPEN_CREATE,
            )
        } else {
            Connection::open(path)
        }
        .map_err(|e| format!("Open DB failed: {}", e))?;

        // 文件数据库：检查是否有保存的 SQLCipher 密码
        if path != MEMORY_DB_URI {
            let passwords = PASSWORD_MAP.lock().unwrap();
            if let Some(pwd) = passwords.get(path) {
                let safe_pwd = pwd.replace('\'', "''");
                conn.execute_batch(&format!("PRAGMA key = '{}';", safe_pwd))
                    .map_err(|e| format!("Set key failed: {}", e))?;
                // 验证密钥是否正确（错误的 key 会导致后续查询失败）
                conn.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))
                    .map_err(|e| format!("Invalid key or corrupted database: {}", e))?;
            }
        }

        cache.insert(path.to_string(), conn);
    }
    let conn = cache.get(path).unwrap();
    f(conn)
}

pub fn resolve_conn(input: &str) -> String {
    if input.is_empty() {
        return MEMORY_DB_URI.to_string();
    }
    let handles = HANDLE_MAP.lock().unwrap();
    handles
        .get(input)
        .cloned()
        .unwrap_or_else(|| input.to_string())
}

pub fn get_handle_map() -> &'static Mutex<HashMap<String, String>> {
    &HANDLE_MAP
}

pub fn get_conn_cache() -> &'static Mutex<HashMap<String, Connection>> {
    &CONN_CACHE
}

pub fn get_handle_counter() -> &'static AtomicUsize {
    &HANDLE_COUNTER
}

pub fn clear_all() {
    CONN_CACHE.lock().unwrap().clear();
    HANDLE_MAP.lock().unwrap().clear();
    PASSWORD_MAP.lock().unwrap().clear();
}
