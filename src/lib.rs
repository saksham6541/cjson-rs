#![deny(unsafe_code)]

pub mod compare;
pub mod error;
pub mod parser;
pub mod printer;

pub use compare::{compare_against_c, compare_against_c_bytes};
pub use error::{ParseError, ParseErrorKind};
pub use parser::parse;
pub use printer::{print, print_unformatted};

/// cJSON-compatible parser entry point.
pub fn from_str(input: &str) -> Result<Value, ParseError> {
    parse(input)
}

/// A JSON value for the cjson-rs port.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
}

impl Default for Value {
    fn default() -> Self {
        Self::Null
    }
}

impl Value {
    pub fn null() -> Self {
        Self::Null
    }

    pub fn bool(value: bool) -> Self {
        Self::Bool(value)
    }

    pub fn number(value: f64) -> Self {
        Self::Number(value)
    }

    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    pub fn array(items: Vec<Value>) -> Self {
        Self::Array(items)
    }

    pub fn object(entries: Vec<(String, Value)>) -> Self {
        Self::Object(entries)
    }

    pub fn new_object() -> Self {
        Self::Object(Vec::new())
    }

    pub fn new_array() -> Self {
        Self::Array(Vec::new())
    }

    pub fn new_string(value: &str) -> Self {
        Self::String(value.to_string())
    }

    pub fn new_number(value: f64) -> Self {
        Self::Number(value)
    }

    pub fn new_bool(value: bool) -> Self {
        Self::Bool(value)
    }

    pub fn new_null() -> Self {
        Self::Null
    }

    pub fn from_str(input: &str) -> Result<Self, ParseError> {
        parse(input)
    }

    pub fn to_string(&self) -> String {
        print_unformatted(self)
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }

    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    pub fn is_true(&self) -> bool {
        matches!(self, Self::Bool(true))
    }

    pub fn is_false(&self) -> bool {
        matches!(self, Self::Bool(false))
    }

    pub fn add_to_object(&mut self, key: &str, value: Value) -> Result<(), ParseError> {
        match self {
            Self::Object(entries) => {
                entries.push((key.to_string(), value));
                Ok(())
            }
            _ => Err(ParseError::type_mismatch("value is not an object")),
        }
    }

    pub fn add_to_array(&mut self, value: Value) -> Result<(), ParseError> {
        match self {
            Self::Array(items) => {
                items.push(value);
                Ok(())
            }
            _ => Err(ParseError::type_mismatch("value is not an array")),
        }
    }

    pub fn get_object_item(&self, key: &str) -> Option<&Value> {
        self.get_object_item_case_insensitive(key)
    }

    pub fn get_object_item_case_sensitive(&self, key: &str) -> Option<&Value> {
        match self {
            Self::Object(entries) => entries
                .iter()
                .find(|(existing_key, _)| existing_key == key)
                .map(|(_, value)| value),
            _ => None,
        }
    }

    pub fn get_object_item_case_insensitive(&self, key: &str) -> Option<&Value> {
        match self {
            Self::Object(entries) => entries
                .iter()
                .find(|(existing_key, _)| existing_key.eq_ignore_ascii_case(key))
                .map(|(_, value)| value),
            _ => None,
        }
    }

    pub fn get_array_item(&self, index: usize) -> Option<&Value> {
        match self {
            Self::Array(items) => items.get(index),
            _ => None,
        }
    }

    pub fn array_size(&self) -> Option<usize> {
        match self {
            Self::Array(items) => Some(items.len()),
            _ => None,
        }
    }

    pub fn array_item(&self, index: usize) -> Option<&Value> {
        self.get_array_item(index)
    }

    pub fn array_item_mut(&mut self, index: usize) -> Option<&mut Value> {
        match self {
            Self::Array(items) => items.get_mut(index),
            _ => None,
        }
    }

    pub fn insert_item_in_array(&mut self, index: usize, item: Value) -> bool {
        match self {
            Self::Array(items) if index <= items.len() => {
                items.insert(index, item);
                true
            }
            _ => false,
        }
    }

    pub fn replace_item_in_array(&mut self, index: usize, item: Value) -> bool {
        match self {
            Self::Array(items) => {
                if let Some(existing) = items.get_mut(index) {
                    *existing = item;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn detach_item_from_array(&mut self, index: usize) -> Option<Value> {
        match self {
            Self::Array(items) if index < items.len() => Some(items.remove(index)),
            _ => None,
        }
    }

    pub fn add_item_to_array(&mut self, item: Value) -> bool {
        match self {
            Self::Array(items) => {
                items.push(item);
                true
            }
            _ => false,
        }
    }

    pub fn add_item_to_object(&mut self, name: impl Into<String>, item: Value) -> bool {
        match self {
            Self::Object(entries) => {
                entries.push((name.into(), item));
                true
            }
            _ => false,
        }
    }

    pub fn replace_item_in_object(&mut self, key: &str, item: Value) -> bool {
        self.replace_item_in_object_case_insensitive(key, item)
    }

    pub fn replace_item_in_object_case_sensitive(&mut self, key: &str, item: Value) -> bool {
        match self {
            Self::Object(entries) => {
                if let Some((_, entry)) = entries.iter_mut().find(|(existing_key, _)| existing_key == key) {
                    *entry = item;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn replace_item_in_object_case_insensitive(&mut self, key: &str, item: Value) -> bool {
        match self {
            Self::Object(entries) => {
                if let Some((_, entry)) = entries.iter_mut().find(|(existing_key, _)| existing_key.eq_ignore_ascii_case(key)) {
                    *entry = item;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn delete_item_from_object(&mut self, key: &str) -> bool {
        self.delete_item_from_object_case_insensitive(key)
    }

    pub fn delete_item_from_object_case_sensitive(&mut self, key: &str) -> bool {
        match self {
            Self::Object(entries) => {
                if let Some(index) = entries.iter().position(|(existing_key, _)| existing_key == key) {
                    entries.remove(index);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn delete_item_from_object_case_insensitive(&mut self, key: &str) -> bool {
        match self {
            Self::Object(entries) => {
                if let Some(index) = entries.iter().position(|(existing_key, _)| existing_key.eq_ignore_ascii_case(key)) {
                    entries.remove(index);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn has_object_item(&self, key: &str) -> bool {
        self.get_object_item(key).is_some()
    }

    pub fn detach_item_from_object(&mut self, key: &str) -> Option<Value> {
        self.detach_item_from_object_case_insensitive(key)
    }

    pub fn detach_item_from_object_case_sensitive(&mut self, key: &str) -> Option<Value> {
        self.detach_item_from_object_with(key, |existing_key| existing_key == key)
    }

    pub fn detach_item_from_object_case_insensitive(&mut self, key: &str) -> Option<Value> {
        self.detach_item_from_object_with(key, |existing_key| existing_key.eq_ignore_ascii_case(key))
    }

    fn detach_item_from_object_with(
        &mut self,
        _key: &str,
        matches: impl Fn(&str) -> bool,
    ) -> Option<Value> {
        match self {
            Self::Object(entries) => entries
                .iter()
                .position(|(existing_key, _)| matches(existing_key))
                .map(|index| entries.remove(index).1),
            _ => None,
        }
    }

    pub fn duplicate(&self, recursive: bool) -> Self {
        if recursive {
            return self.clone();
        }

        match self {
            Self::Array(_) => Self::Array(Vec::new()),
            Self::Object(_) => Self::Object(Vec::new()),
            _ => self.clone(),
        }
    }

    pub fn compare(&self, other: &Self, case_sensitive: bool) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Bool(left), Self::Bool(right)) => left == right,
            (Self::Number(left), Self::Number(right)) => {
                if left.is_nan() || right.is_nan() {
                    false
                } else {
                    let max = left.abs().max(right.abs());
                    (left - right).abs() <= max * f64::EPSILON
                }
            }
            (Self::String(left), Self::String(right)) => left == right,
            (Self::Array(left), Self::Array(right)) => {
                left.len() == right.len()
                    && left
                        .iter()
                        .zip(right)
                        .all(|(left, right)| left.compare(right, case_sensitive))
            }
            (Self::Object(left), Self::Object(right)) => {
                left.len() == right.len()
                    && left.iter().all(|(key, value)| {
                        right.iter().find(|(other_key, _)| {
                            if case_sensitive {
                                other_key == key
                            } else {
                                other_key.eq_ignore_ascii_case(key)
                            }
                        })
                        .is_some_and(|(_, other_value)| value.compare(other_value, case_sensitive))
                    })
            }
            _ => false,
        }
    }

    pub fn to_string_pretty(&self) -> String {
        printer::print(self)
    }

    pub fn as_clamped_int(&self) -> Option<i32> {
        match self {
            Self::Number(n) => Some(clamp_to_valueint(*n)),
            _ => None,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

fn clamp_to_valueint(n: f64) -> i32 {
    if !n.is_finite() {
        return 0;
    }

    if n >= i32::MAX as f64 {
        i32::MAX
    } else if n <= i32::MIN as f64 {
        i32::MIN
    } else {
        n as i32
    }
}

pub mod value {
    pub use crate::Value;
}

#[allow(non_snake_case)]
pub fn cJSON_CreateObject() -> Value {
    Value::new_object()
}

#[allow(non_snake_case)]
pub fn cJSON_CreateArray() -> Value {
    Value::new_array()
}

#[allow(non_snake_case)]
pub fn cJSON_CreateString(value: impl Into<String>) -> Value {
    Value::string(value)
}

#[allow(non_snake_case)]
pub fn cJSON_CreateNumber(value: f64) -> Value {
    Value::number(value)
}

#[allow(non_snake_case)]
pub fn cJSON_CreateBool(value: bool) -> Value {
    Value::bool(value)
}

#[allow(non_snake_case)]
pub fn cJSON_CreateNull() -> Value {
    Value::new_null()
}

#[allow(non_snake_case)]
pub fn cJSON_Parse(input: &str) -> Result<Value, ParseError> {
    Value::from_str(input)
}

#[allow(non_snake_case)]
pub fn cJSON_Print(value: &Value) -> String {
    value.to_string_pretty()
}

#[allow(non_snake_case)]
pub fn cJSON_PrintUnformatted(value: &Value) -> String {
    value.to_string()
}

#[allow(non_snake_case)]
pub fn cJSON_AddItemToObject(object: &mut Value, name: &str, item: Value) -> bool {
    object.add_to_object(name, item).is_ok()
}

#[allow(non_snake_case)]
pub fn cJSON_AddItemToArray(array: &mut Value, item: Value) -> bool {
    array.add_to_array(item).is_ok()
}

#[allow(non_snake_case)]
pub fn cJSON_GetObjectItem<'a>(object: &'a Value, name: &str) -> Option<&'a Value> {
    object.get_object_item(name)
}

#[allow(non_snake_case)]
pub fn cJSON_GetArrayItem<'a>(array: &'a Value, index: usize) -> Option<&'a Value> {
    array.get_array_item(index)
}

#[allow(non_snake_case)]
pub fn cJSON_IsNull(value: &Value) -> bool {
    value.is_null()
}

#[allow(non_snake_case)]
pub fn cJSON_IsBool(value: &Value) -> bool {
    value.is_bool()
}

#[allow(non_snake_case)]
pub fn cJSON_IsNumber(value: &Value) -> bool {
    value.is_number()
}

#[allow(non_snake_case)]
pub fn cJSON_IsString(value: &Value) -> bool {
    value.is_string()
}

#[allow(non_snake_case)]
pub fn cJSON_IsArray(value: &Value) -> bool {
    value.is_array()
}

#[allow(non_snake_case)]
pub fn cJSON_IsObject(value: &Value) -> bool {
    value.is_object()
}
