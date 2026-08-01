use crate::Value;

pub fn print(value: &Value) -> String {
    print_value(value, true, 0)
}

pub fn print_unformatted(value: &Value) -> String {
    print_value(value, false, 0)
}

fn print_value(value: &Value, pretty: bool, depth: usize) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(true) => "true".to_string(),
        Value::Bool(false) => "false".to_string(),
        Value::Number(number) => format_number(*number),
        Value::String(text) => format_string(text),
        Value::Array(items) => {
            if items.is_empty() {
                "[]".to_string()
            } else if pretty {
                let inner = items
                    .iter()
                    .map(|item| {
                        format!(
                            "{}{}",
                            indent(depth + 1),
                            print_value(item, true, depth + 1)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",\n");
                format!("[\n{}\n{}]", inner, indent(depth))
            } else {
                format!(
                    "[{}]",
                    items
                        .iter()
                        .map(|item| print_value(item, false, depth))
                        .collect::<Vec<_>>()
                        .join(",")
                )
            }
        }
        Value::Object(entries) => {
            if entries.is_empty() {
                "{}".to_string()
            } else if pretty {
                let inner = entries
                    .iter()
                    .map(|(key, value)| {
                        format!(
                            "{}{}: {}",
                            indent(depth + 1),
                            format_string(key),
                            print_value(value, true, depth + 1)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",\n");
                format!("{{\n{}\n{}}}", inner, indent(depth))
            } else {
                format!(
                    "{{{}}}",
                    entries
                        .iter()
                        .map(|(key, value)| format!(
                            "{}:{}",
                            format_string(key),
                            print_value(value, false, depth)
                        ))
                        .collect::<Vec<_>>()
                        .join(",")
                )
            }
        }
    }
}

fn format_number(number: f64) -> String {
    if !number.is_finite() {
        return "null".to_string();
    }

    if number == 0.0 && number.is_sign_negative() {
        return "-0".to_string();
    }

    if number.fract() == 0.0 && number >= i64::MIN as f64 && number <= i64::MAX as f64 {
        return format!("{}", number as i64);
    }

    if number.abs() >= 1e15 || number.abs() <= 1e-6 {
        let scientific = format!("{number:.15e}");
        let mut parts = scientific.split('e');
        let mantissa = parts.next().unwrap_or_default();
        let exponent = parts.next().unwrap_or_default();
        let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
        if mantissa.is_empty() {
            return format!("0e{exponent}");
        }
        return format!("{mantissa}e{exponent}");
    }

    for precision in [15, 17] {
        let candidate = format!("{number:.precision$}");
        if let Ok(parsed) = candidate.parse::<f64>() {
            if compare_double(number, parsed) {
                return candidate;
            }
        }
    }

    number.to_string()
}

fn compare_double(left: f64, right: f64) -> bool {
    let max = left.abs().max(right.abs());
    (left - right).abs() <= max * f64::EPSILON
}

fn format_string(value: &str) -> String {
    let mut escaped = String::from('"');
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{0008}' => escaped.push_str("\\b"),
            '\u{000C}' => escaped.push_str("\\f"),
            ch if ch.is_control() => escaped.push_str(&format!("\\u{:04x}", ch as u32)),
            _ => escaped.push(ch),
        }
    }
    escaped.push('"');
    escaped
}

fn indent(depth: usize) -> String {
    "  ".repeat(depth)
}
