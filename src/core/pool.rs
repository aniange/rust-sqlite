use rusqlite::Connection;
use std::collections::HashMap;

pub struct ConnectionPool {
    cache: HashMap<String, Connection>,
}

impl ConnectionPool {
    pub fn new() -> Self {
        Self { cache: HashMap::new() }
    }

    pub fn get(&mut self, path: &str) -> Result<&mut Connection, rusqlite::Error> {
        if !self.cache.contains_key(path) {
            let conn = Connection::open(path)?;
            self.cache.insert(path.to_string(), conn);
        }
        Ok(self.cache.get_mut(path).unwrap())
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }

    pub fn remove(&mut self, path: &str) {
        self.cache.remove(path);
    }
}
