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

    pub fn array_size(&self) -> Option<usize> {
        match self {
            Self::Array(items) => Some(items.len()),
            _ => None,
        }
    }

    pub fn array_item(&self, index: usize) -> Option<&Value> {
        match self {
            Self::Array(items) => items.get(index),
            _ => None,
        }
    }

    pub fn array_item_mut(&mut self, index: usize) -> Option<&mut Value> {
        match self {
            Self::Array(items) => items.get_mut(index),
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

    pub fn add_item_to_object(&mut self, name: impl Into<String>, item: Value) -> bool {
        match self {
            Self::Object(entries) => {
                entries.push((name.into(), item));
                true
            }
            _ => false,
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

    pub fn has_object_item(&self, name: &str) -> bool {
        self.get_object_item(name).is_some()
    }

    pub fn detach_item_from_object(&mut self, name: &str) -> Option<Value> {
        self.detach_item_from_object_case_insensitive(name)
    }

    pub fn detach_item_from_object_case_sensitive(&mut self, name: &str) -> Option<Value> {
        self.detach_item_from_object_with(name, |key| key == name)
    }

    pub fn detach_item_from_object_case_insensitive(&mut self, name: &str) -> Option<Value> {
        self.detach_item_from_object_with(name, |key| key.eq_ignore_ascii_case(name))
    }

    fn detach_item_from_object_with(
        &mut self,
        _name: &str,
        matches: impl Fn(&str) -> bool,
    ) -> Option<Value> {
        match self {
            Self::Object(entries) => entries
                .iter()
                .position(|(key, _)| matches(key))
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
                        let other = right.iter().find(|(other_key, _)| {
                            if case_sensitive {
                                other_key == key
                            } else {
                                other_key.eq_ignore_ascii_case(key)
                            }
                        });
                        other.is_some_and(|(_, other_value)| {
                            value.compare(other_value, case_sensitive)
                        })
                    })
            }
            _ => false,
        }
    }

    /// Equivalent of cJSON's `valueint`: a 32-bit integer view of a number,
    /// clamped to `i32::MIN`/`i32::MAX` on overflow rather than widened to a
    /// larger Rust integer type. cJSON always keeps both `valuedouble` and a
    /// clamped `valueint` in sync for every parsed number; `Value::Number`
    /// only stores the `f64`, so this derives the clamped view on demand.
    /// See DECISIONS.md for why the clamp (not a wider type) was kept.
    pub fn as_clamped_int(&self) -> Option<i32> {
        match self {
            Self::Number(n) => Some(clamp_to_valueint(*n)),
            _ => None,
        }
    }
}

fn clamp_to_valueint(n: f64) -> i32 {
    // The parser never produces NaN/Infinity (parse_number only accepts
    // JSON's numeric grammar), but guard anyway since this is a public API.
    if !n.is_finite() {
        return 0;
    }
    if n >= i32::MAX as f64 {
        i32::MAX
    } else if n <= i32::MIN as f64 {
        i32::MIN
    } else {
        // Truncates toward zero, matching C's `(int)double` cast.
        n as i32
    }
}
