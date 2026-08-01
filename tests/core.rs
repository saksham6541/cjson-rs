use std::env;

use hackathon::{compare_against_c, parse, print, print_unformatted, value::Value};

#[test]
fn parses_objects_arrays_and_scalars() {
    let value =
        parse(r#"{"name":"Ada","active":true,"score":42,"items":[1,null,{"x":2}] }"#).unwrap();
    assert!(matches!(value, Value::Object(_)));
    let rendered = print(&value);
    assert!(rendered.contains("\"name\": \"Ada\""));
    assert!(rendered.contains("\"active\": true"));
    assert!(rendered.contains("\"items\""));
}

#[test]
fn preserves_first_duplicate_object_key() {
    let value = parse(r#"{"first":1,"first":2}"#).unwrap();
    let item = value.get_object_item("first").unwrap();
    assert!(matches!(item, Value::Number(n) if (*n - 1.0).abs() < f64::EPSILON));
}

#[test]
fn round_trips_pretty_and_compact_output() {
    let value = parse(r#"{"a":[1,2,3],"b":{"c":true}}"#).unwrap();
    let pretty = print(&value);
    let compact = print_unformatted(&value);
    assert!(pretty.contains("\n"));
    assert!(!compact.contains('\n'));
    assert!(compact.contains("\"a\": [1, 2, 3]"));
}

#[test]
fn prints_large_numbers_in_scientific_notation() {
    let rendered = print_unformatted(&Value::Number(1e20));
    assert_eq!(rendered, "1e20");
}

#[test]
fn object_lookup_is_case_insensitive_by_default() {
    let value = parse(r#"{"Name": 7}"#).unwrap();
    assert!(value.get_object_item("name").is_some());
    assert!(value.get_object_item_case_sensitive("name").is_none());
}

#[test]
fn parses_literals_after_non_ascii_text() {
    let value = parse(r#"{"emoji":"🎉","flag":true}"#).unwrap();
    assert!(matches!(
        value.get_object_item("flag"),
        Some(Value::Bool(true))
    ));
}

#[test]
fn rejects_non_json_whitespace() {
    assert!(parse("{\"a\":1\u{00A0},\"b\":2}").is_err());
}

#[test]
fn compares_against_c_reference_from_an_external_cwd() {
    let original_dir = env::current_dir().unwrap();
    let temp_dir = env::temp_dir().join("hackathon-c-reference-check");
    let _ = std::fs::create_dir_all(&temp_dir);

    env::set_current_dir(&temp_dir).unwrap();
    let result = compare_against_c(r#"{"a":[1,2,3]}"#);
    env::set_current_dir(original_dir).unwrap();

    assert!(
        result.is_ok(),
        "expected comparison to succeed, got {result:?}"
    );
    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn object_mutation_helpers_use_case_insensitive_lookup_by_default() {
    let mut value = parse(r#"{"Name":1,"other":2}"#).unwrap();
    assert!(value.replace_item_in_object("name", Value::Number(5.0)));
    assert!(value.delete_item_from_object("OTHER"));
    assert!(
        matches!(value.get_object_item("Name"), Some(Value::Number(n)) if (*n - 5.0).abs() < f64::EPSILON)
    );
    assert!(value.get_object_item("other").is_none());
}

#[test]
fn object_mutation_helpers_support_explicit_case_sensitive_lookup() {
    let mut value = parse(r#"{"Name":1,"name":2}"#).unwrap();
    assert!(value.replace_item_in_object_case_sensitive("name", Value::Number(9.0)));
    assert!(
        matches!(value.get_object_item_case_sensitive("name"), Some(Value::Number(n)) if (*n - 9.0).abs() < f64::EPSILON)
    );
    assert!(value.get_object_item_case_sensitive("Name").is_some());
}
