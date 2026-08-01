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

    pub fn add_item_to_array(&mut self, item: Value) {
        match self {
            Self::Array(items) => items.push(item),
            _ => panic!("add_item_to_array requires an array value"),
        }
    }

    pub fn add_item_to_object(&mut self, name: impl Into<String>, item: Value) {
        match self {
            Self::Object(entries) => entries.push((name.into(), item)),
            _ => panic!("add_item_to_object requires an object value"),
        }
    }

    pub fn replace_item_in_object(&mut self, name: &str, item: Value) -> bool {
        self.replace_item_in_object_case_insensitive(name, item)
    }

    pub fn replace_item_in_object_case_sensitive(&mut self, name: &str, item: Value) -> bool {
        match self {
            Self::Object(entries) => {
                if let Some((_, entry)) = entries.iter_mut().find(|(key, _)| key == name) {
                    *entry = item;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn replace_item_in_object_case_insensitive(&mut self, name: &str, item: Value) -> bool {
        match self {
            Self::Object(entries) => {
                if let Some((_, entry)) = entries
                    .iter_mut()
                    .find(|(key, _)| key.eq_ignore_ascii_case(name))
                {
                    *entry = item;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn delete_item_from_object(&mut self, name: &str) -> bool {
        self.delete_item_from_object_case_insensitive(name)
    }

    pub fn delete_item_from_object_case_sensitive(&mut self, name: &str) -> bool {
        match self {
            Self::Object(entries) => {
                if let Some(index) = entries.iter().position(|(key, _)| key == name) {
                    entries.remove(index);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn delete_item_from_object_case_insensitive(&mut self, name: &str) -> bool {
        match self {
            Self::Object(entries) => {
                if let Some(index) = entries
                    .iter()
                    .position(|(key, _)| key.eq_ignore_ascii_case(name))
                {
                    entries.remove(index);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn get_array_size(&self) -> usize {
        match self {
            Self::Array(items) => items.len(),
            _ => 0,
        }
    }

    pub fn get_array_item(&self, index: usize) -> Option<&Value> {
        match self {
            Self::Array(items) => items.get(index),
            _ => None,
        }
    }

    pub fn has_object_item(&self, name: &str) -> bool {
        self.get_object_item(name).is_some()
    }

    pub fn get_string_value(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn get_number_value(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(*value),
            _ => None,
        }
    }

    pub fn get_object_item(&self, name: &str) -> Option<&Value> {
        self.get_object_item_case_insensitive(name)
    }

    pub fn get_object_item_case_sensitive(&self, name: &str) -> Option<&Value> {
        match self {
            Self::Object(entries) => entries
                .iter()
                .find(|(key, _)| key == name)
                .map(|(_, value)| value),
            _ => None,
        }
    }

    pub fn get_object_item_case_insensitive(&self, name: &str) -> Option<&Value> {
        match self {
            Self::Object(entries) => entries
                .iter()
                .find(|(key, _)| key.eq_ignore_ascii_case(name))
                .map(|(_, value)| value),
            _ => None,
        }
    }
}
