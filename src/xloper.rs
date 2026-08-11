use xll_rs::types::*;

pub unsafe fn xloper_to_string_grid(op: *const XLOPER12) -> Option<Vec<Vec<String>>> {
    if op.is_null() || (*op).base_type() != XLTYPE_MULTI {
        return None;
    }
    let arr = &(*op).val.array;
    let rows = arr.rows as usize;
    let cols = arr.columns as usize;
    if rows == 0 || cols == 0 {
        return Some(Vec::new());
    }
    if arr.lparray.is_null() {
        return None;
    }
    let total = rows * cols;
    if total > 1_000_000 {
        return None;
    }

    let mut result = Vec::with_capacity(rows);
    for r in 0..rows {
        let mut row = Vec::with_capacity(cols);
        for c in 0..cols {
            let cell = &*arr.lparray.add(r * cols + c);
            let s = match cell.base_type() {
                XLTYPE_STR => cell.as_string().unwrap_or_default(),
                XLTYPE_NUM => {
                    let n = cell.as_f64().unwrap_or(0.0);
                    if n.fract() == 0.0 { format!("{:.0}", n) } else { n.to_string() }
                }
                XLTYPE_INT => format!("{:.0}", cell.as_f64().unwrap_or(0.0)),
                XLTYPE_BOOL => cell.as_bool().map(|b| if b { "1".to_string() } else { "0".to_string() }).unwrap_or_default(),
                _ => String::new(),
            };
            row.push(s);
        }
        result.push(row);
    }
    Some(result)
}

pub unsafe fn xloper_to_string_list(op: *const XLOPER12) -> Option<Vec<String>> {
    match (*op).base_type() {
        XLTYPE_MULTI => {
            let arr = &(*op).val.array;
            let rows = arr.rows as usize;
            let cols = arr.columns as usize;
            if rows == 0 || cols == 0 {
                return Some(Vec::new());
            }
            let effective_rows = if rows > 1 && cols > 1 { 1 } else { rows };
            let mut result = Vec::with_capacity(effective_rows * cols);
            for r in 0..effective_rows {
                for c in 0..cols {
                    let cell = &*arr.lparray.add(r * cols + c);
                    let s = match cell.base_type() {
                        XLTYPE_STR => cell.as_string().unwrap_or_default(),
                        XLTYPE_NUM => {
                            let n = cell.as_f64().unwrap_or(0.0);
                            if n.fract() == 0.0 { format!("{:.0}", n) } else { n.to_string() }
                        }
                        XLTYPE_INT => format!("{:.0}", cell.as_f64().unwrap_or(0.0)),
                        XLTYPE_BOOL => cell.as_bool().map(|b| b.to_string()).unwrap_or_default(),
                        _ => String::new(),
                    };
                    result.push(s);
                }
            }
            Some(result)
        }
        XLTYPE_STR => Some(vec![(*op).as_string().unwrap_or_default()]),
        XLTYPE_NUM => {
            let n = (*op).as_f64().unwrap_or(0.0);
            Some(vec![if n.fract() == 0.0 { format!("{:.0}", n) } else { n.to_string() }])
        }
        XLTYPE_INT => Some(vec![format!("{:.0}", (*op).as_f64().unwrap_or(0.0))]),
        XLTYPE_BOOL => Some(vec![(*op).as_bool().map(|b| b.to_string()).unwrap_or_default()]),
        _ => None,
    }
}

pub unsafe fn extract_conn_str(op: *mut XLOPER12) -> Option<String> {
    if (*op).base_type() == XLTYPE_MISSING {
        Some(crate::conn::MEMORY_DB_URI.to_string())
    } else {
        match (*op).as_string() {
            Some(s) if !s.is_empty() => Some(s),
            _ => Some(crate::conn::MEMORY_DB_URI.to_string()),
        }
    }
}

pub fn sqlite_value_to_xloper(value: &rusqlite::types::Value) -> XLOPER12 {
    use rusqlite::types::Value;
    match value {
        Value::Null => XLOPER12::from_str(""),
        Value::Integer(i) => XLOPER12::from_f64(*i as f64),
        Value::Real(f) => XLOPER12::from_f64(*f),
        Value::Text(s) => XLOPER12::from_str(s),
        Value::Blob(b) => {
            let hex = b.iter().map(|byte| format!("{:02X}", byte)).collect::<String>();
            XLOPER12::from_str(&hex)
        }
    }
}
