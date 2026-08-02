use crate::Value;

pub fn print(value: &Value) -> String {
    let mut out = String::new();
    write_value(&mut out, value, true, 0);
    out
}

pub fn print_unformatted(value: &Value) -> String {
    let mut out = String::new();
    write_value(&mut out, value, false, 0);
    out
}

fn write_value(out: &mut String, value: &Value, pretty: bool, depth: usize) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(number) => out.push_str(&format_number(*number)),
        Value::String(text) => write_string(out, text),
        Value::Array(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return;
            }
            if pretty {
                out.push('[');
                out.push('\n');
                for (i, item) in items.iter().enumerate() {
                    write_indent(out, depth + 1);
                    write_value(out, item, true, depth + 1);
                    if i + 1 != items.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                write_indent(out, depth);
                out.push(']');
            } else {
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write_value(out, item, false, depth);
                }
                out.push(']');
            }
        }
        Value::Object(entries) => {
            if entries.is_empty() {
                out.push_str("{}");
                return;
            }
            if pretty {
                out.push('{');
                out.push('\n');
                for (i, (key, value)) in entries.iter().enumerate() {
                    write_indent(out, depth + 1);
                    write_string(out, key);
                    out.push_str(": ");
                    write_value(out, value, true, depth + 1);
                    if i + 1 != entries.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                write_indent(out, depth);
                out.push('}');
            } else {
                out.push('{');
                for (i, (key, value)) in entries.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write_string(out, key);
                    out.push(':');
                    write_value(out, value, false, depth);
                }
                out.push('}');
            }
        }
    }
}

fn write_indent(out: &mut String, depth: usize) {
    for _ in 0..depth {
        out.push_str("  ");
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

fn write_string(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000C}' => out.push_str("\\f"),
            ch if ch.is_control() => {
                use std::fmt::Write;
                let _ = write!(out, "\\u{:04x}", ch as u32);
            }
            _ => out.push(ch),
        }
    }
    out.push('"');
}
