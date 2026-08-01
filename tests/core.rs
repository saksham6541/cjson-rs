use std::env;
use std::path::PathBuf;

use hackathon::{
    compare_against_c,
    parse,
    print,
    print_unformatted,
    value::Value,
    cJSON_AddItemToArray,
    cJSON_AddItemToObject,
    cJSON_CreateArray,
    cJSON_CreateBool,
    cJSON_CreateNull,
    cJSON_CreateNumber,
    cJSON_CreateObject,
    cJSON_CreateString,
    cJSON_GetArrayItem,
    cJSON_GetObjectItem,
    cJSON_IsArray,
    cJSON_IsBool,
    cJSON_IsNull,
    cJSON_IsNumber,
    cJSON_IsObject,
    cJSON_IsString,
    cJSON_Parse,
    cJSON_Print,
    cJSON_PrintUnformatted,
    ParseErrorKind,
};

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
    assert_eq!(compact, r#"{"a":[1,2,3],"b":{"c":true}}"#);
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
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let potential_paths = [
        manifest_dir.join("../cJSON/tests"),
        manifest_dir.join("original/cJSON/tests"),
        manifest_dir.join("../original/cJSON/tests"),
    ];
    let exists = potential_paths.iter().any(|path| path.is_dir());
    if !exists {
        eprintln!("Skipping C reference comparison because upstream cJSON tests are not available.");
        return;
    }

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

#[test]
fn rejects_deeply_nested_input_at_the_parser_limit() {
    let input = format!("{}{}{}", "[".repeat(1001), "1", "]".repeat(1001));
    assert!(parse(&input).is_err());
}

#[test]
fn supports_array_manipulation_helpers() {
    let mut value = Value::array(vec![Value::number(1.0), Value::number(3.0)]);
    assert_eq!(value.array_size(), Some(2));
    assert!(value.insert_item_in_array(1, Value::number(2.0)));
    assert!(value.replace_item_in_array(2, Value::number(4.0)));
    assert_eq!(value.detach_item_from_array(0), Some(Value::number(1.0)));
    assert_eq!(value.array_item(0), Some(&Value::number(2.0)));
    assert!(!value.insert_item_in_array(10, Value::null()));
}

#[test]
fn supports_object_presence_detach_and_duplicate_helpers() {
    let mut value = parse(r#"{"Name":1,"other":2}"#).unwrap();
    let full_duplicate = value.duplicate(true);
    assert!(value.has_object_item("name"));
    assert_eq!(
        value.detach_item_from_object_case_sensitive("Name"),
        Some(Value::number(1.0))
    );
    assert_eq!(
        value.detach_item_from_object("OTHER"),
        Some(Value::number(2.0))
    );
    assert_eq!(value.duplicate(false), Value::Object(Vec::new()));
    assert_eq!(
        full_duplicate,
        Value::Object(vec![
            ("Name".to_string(), Value::number(1.0)),
            ("other".to_string(), Value::number(2.0))
        ])
    );
}

#[test]
fn supports_type_predicates_and_semantic_comparison() {
    let left = parse(r#"{"Name":[1,true]}"#).unwrap();
    let right = parse(r#"{"name":[1,true]}"#).unwrap();
    assert!(left.is_object());
    assert!(left.compare(&right, false));
    assert!(!left.compare(&right, true));
    assert!(Value::bool(true).is_true());
    assert!(Value::bool(false).is_false());
    assert!(Value::null().is_null());
    assert!(Value::number(1.0).is_number());
    assert!(Value::string("x").is_string());
}

#[test]
fn value_new_constructors_and_predicates_work() {
    let object = Value::new_object();
    let array = Value::new_array();
    let string = Value::new_string("hello");
    let number = Value::new_number(3.14);
    let boolean = Value::new_bool(true);
    let null = Value::new_null();

    assert!(object.is_object());
    assert!(array.is_array());
    assert!(string.is_string());
    assert!(number.is_number());
    assert!(boolean.is_bool());
    assert!(null.is_null());
}

#[test]
fn value_manipulation_and_pretty_printing_work() {
    let mut object = Value::new_object();
    assert!(object.add_to_object("key", Value::new_string("value")).is_ok());
    assert_eq!(object.get_object_item("key"), Some(&Value::new_string("value")));

    let mut array = Value::new_array();
    assert!(array.add_to_array(Value::new_number(2.0)).is_ok());
    assert_eq!(array.get_array_item(0), Some(&Value::new_number(2.0)));
    assert!(array.get_array_item(1).is_none());

    let pretty = object.to_string_pretty();
    assert!(pretty.contains("\"key\": \"value\""));
}

#[test]
fn from_str_and_to_string_round_trip() {
    let input = r#"{"emoji":"🚀","value":-0.0,"active":false}"#;
    let value = Value::from_str(input).unwrap();
    assert!(value.is_object());
    assert_eq!(value.to_string(), r#"{"emoji":"🚀","value":-0,"active":false}"#);
    assert!(value.to_string_pretty().contains("\"emoji\": \"🚀\""));
}

#[test]
fn value_print_negative_zero_and_nonfinite_values() {
    assert_eq!(Value::new_number(-0.0).to_string(), "-0");
    assert_eq!(Value::new_number(f64::NAN).to_string(), "null");
    assert_eq!(Value::new_number(f64::INFINITY).to_string(), "null");
    assert_eq!(Value::new_number(f64::NEG_INFINITY).to_string(), "null");
}

#[test]
fn cjson_wrapper_parse_and_print_helpers_work() {
    let value = cJSON_Parse(r#"{"test":true}"#).unwrap();
    assert!(cJSON_IsObject(&value));
    let output = cJSON_PrintUnformatted(&value);
    assert_eq!(output, r#"{"test":true}"#);
    let pretty = cJSON_Print(&value);
    assert!(pretty.contains("\n"));
}

#[test]
fn value_manipulation_errors_are_reported_for_wrong_types() {
    let mut scalar = Value::new_number(1.0);
    let object_error = scalar.add_to_object("key", Value::new_null());
    assert!(object_error.is_err());
    assert_eq!(object_error.unwrap_err().kind, ParseErrorKind::TypeMismatch);

    let array_error = scalar.add_to_array(Value::new_null());
    assert!(array_error.is_err());
    assert_eq!(array_error.unwrap_err().kind, ParseErrorKind::TypeMismatch);
}

#[test]
fn cjson_create_add_get_helpers_work_for_object_and_array() {
    let mut object = cJSON_CreateObject();
    assert!(cJSON_IsObject(&object));
    assert!(!cJSON_IsArray(&object));

    let mut array = cJSON_CreateArray();
    assert!(cJSON_IsArray(&array));
    assert!(!cJSON_IsNull(&array));

    assert!(cJSON_AddItemToObject(
        &mut object,
        "name",
        cJSON_CreateString("Rust")
    ));
    assert!(cJSON_AddItemToObject(
        &mut object,
        "value",
        cJSON_CreateNumber(123.0)
    ));
    assert!(cJSON_AddItemToObject(&mut object, "active", cJSON_CreateBool(true)));
    assert!(cJSON_AddItemToObject(&mut object, "missing", cJSON_CreateNull()));

    assert!(cJSON_AddItemToArray(&mut array, cJSON_CreateString("first")));
    assert!(cJSON_AddItemToArray(&mut array, cJSON_CreateNumber(2.0)));
    assert!(cJSON_AddItemToArray(&mut array, cJSON_CreateBool(false)));

    let name_item = cJSON_GetObjectItem(&object, "name");
    assert!(matches!(name_item, Some(Value::String(value)) if value == "Rust"));
    assert!(cJSON_GetObjectItem(&object, "VALUE").is_some());
    assert!(cJSON_GetObjectItem(&object, "active").is_some());
    assert!(cJSON_GetObjectItem(&object, "missing").is_some());

    assert!(matches!(cJSON_GetArrayItem(&array, 0), Some(Value::String(_))));
    assert!(matches!(cJSON_GetArrayItem(&array, 1), Some(Value::Number(n)) if (*n - 2.0).abs() < f64::EPSILON));
    assert!(matches!(cJSON_GetArrayItem(&array, 2), Some(Value::Bool(false))));
    assert!(cJSON_GetArrayItem(&array, 3).is_none());
}

#[test]
fn cjson_type_predicates_cover_all_value_kinds() {
    assert!(cJSON_IsNull(&cJSON_CreateNull()));
    assert!(cJSON_IsBool(&cJSON_CreateBool(false)));
    assert!(cJSON_IsNumber(&cJSON_CreateNumber(0.0)));
    assert!(cJSON_IsString(&cJSON_CreateString("x")));
    assert!(cJSON_IsArray(&cJSON_CreateArray()));
    assert!(cJSON_IsObject(&cJSON_CreateObject()));
}

#[test]
fn cjson_add_item_helpers_reject_incorrect_target_types() {
    let mut scalar = cJSON_CreateNumber(1.0);
    assert!(!cJSON_AddItemToObject(&mut scalar, "x", cJSON_CreateNull()));
    assert!(!cJSON_AddItemToArray(&mut scalar, cJSON_CreateNull()));
}
