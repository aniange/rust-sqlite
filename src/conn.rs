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
}

pub fn with_conn<T, F>(path: &str, f: F) -> Result<T, String>
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
        }.map_err(|e| format!("Open DB failed: {}", e))?;
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
    handles.get(input).cloned().unwrap_or_else(|| input.to_string())
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
}
