use super::*;

// ========== infer_column_type tests ==========

#[test]
fn test_infer_column_type_empty() {
    assert_eq!(infer_column_type(&[]), "TEXT");
}

#[test]
fn test_infer_column_type_all_empty_strings() {
    assert_eq!(infer_column_type(&["".to_string(), "".to_string()]), "TEXT");
}

#[test]
fn test_infer_column_type_integer() {
    assert_eq!(infer_column_type(&["123".to_string(), "456".to_string()]), "INTEGER");
}

#[test]
fn test_infer_column_type_integer_with_empty() {
    assert_eq!(infer_column_type(&["123".to_string(), "".to_string(), "456".to_string()]), "INTEGER");
}

#[test]
fn test_infer_column_type_integer_negative() {
    assert_eq!(infer_column_type(&["-123".to_string(), "0".to_string()]), "INTEGER");
}

#[test]
fn test_infer_column_type_real() {
    assert_eq!(infer_column_type(&["3.14".to_string(), "2.71".to_string()]), "REAL");
}

#[test]
fn test_infer_column_type_real_mixed_with_int() {
    assert_eq!(infer_column_type(&["3.14".to_string(), "2".to_string()]), "REAL");
}

#[test]
fn test_infer_column_type_text() {
    assert_eq!(infer_column_type(&["hello".to_string(), "world".to_string()]), "TEXT");
}

#[test]
fn test_infer_column_type_text_mixed_with_int() {
    assert_eq!(infer_column_type(&["123".to_string(), "abc".to_string()]), "TEXT");
}

// ========== make_valid_columns tests ==========

#[test]
fn test_make_valid_columns_basic() {
    let input = vec!["id".to_string(), "name".to_string()];
    let result = make_valid_columns(input, 2);
    assert_eq!(result, vec!["id", "name"]);
}

#[test]
fn test_make_valid_columns_with_quotes() {
    let input = vec!["\"id\"".to_string(), "\"name\"".to_string()];
    let result = make_valid_columns(input, 2);
    assert_eq!(result, vec!["id", "name"]);
}

#[test]
fn test_make_valid_columns_with_empty() {
    let input = vec!["id".to_string(), "".to_string()];
    let result = make_valid_columns(input, 2);
    assert_eq!(result, vec!["id", "col_2"]);
}

#[test]
fn test_make_valid_columns_pads_to_expected() {
    let input = vec!["a".to_string()];
    let result = make_valid_columns(input, 3);
    assert_eq!(result, vec!["a", "col_2", "col_3"]);
}

#[test]
fn test_make_valid_columns_truncates_to_expected() {
    let input = vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()];
    let result = make_valid_columns(input, 2);
    assert_eq!(result, vec!["a", "b"]);
}

// ========== normalize_sql_type tests ==========

#[test]
fn test_normalize_sql_type_integer_aliases() {
    assert_eq!(normalize_sql_type("INT"), "INTEGER");
    assert_eq!(normalize_sql_type("bigint"), "INTEGER");
    assert_eq!(normalize_sql_type("  INTEGER  "), "INTEGER");
}

#[test]
fn test_normalize_sql_type_real_aliases() {
    assert_eq!(normalize_sql_type("FLOAT"), "REAL");
    assert_eq!(normalize_sql_type("DOUBLE"), "REAL");
}

#[test]
fn test_normalize_sql_type_text_aliases() {
    assert_eq!(normalize_sql_type("VARCHAR"), "TEXT");
    assert_eq!(normalize_sql_type("STRING"), "TEXT");
}

#[test]
fn test_normalize_sql_type_unknown() {
    assert_eq!(normalize_sql_type("UNKNOWN"), "TEXT");
    assert_eq!(normalize_sql_type(""), "TEXT");
}
