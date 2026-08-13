#![allow(clippy::not_unsafe_ptr_arg_deref)]
use xll_rs::types::*;

pub fn error_to_xloper(err: &str) -> XLOPER12 {
    let upper = err.to_uppercase();
    if err.contains("Open DB failed") || err.contains("unable to open")
        || upper.contains("NO SUCH TABLE") || upper.contains("NO SUCH COLUMN")
    {
        XLOPER12::from_err(XLERR_REF)
    } else if err.contains("Prepare failed") || err.contains("syntax error")
        || err.contains("unrecognized token") || err.contains("Connect failed")
    {
        XLOPER12::from_err(XLERR_NAME)
    } else {
        XLOPER12::from_err(XLERR_VALUE)
    }
}
