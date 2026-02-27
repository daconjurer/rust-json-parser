use std::{collections::HashMap, fmt};

fn escape_json_string(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\t' => result.push_str("\\t"),
            '\r' => result.push_str("\\r"),
            '\u{0008}' => result.push_str("\\b"),
            '\u{000C}' => result.push_str("\\f"),
            _ => result.push(c),
        }
    }
    result
}

/// Represents a parsed JSON value.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    /// A JSON string (e.g. `"hello"`).
    String(String),
    /// A JSON number, stored as `f64` (e.g. `42`, `3.14`).
    Number(f64),
    /// A JSON boolean (`true` or `false`).
    Boolean(bool),
    /// The JSON `null` literal.
    Null,
    /// An ordered JSON array of values (e.g. `[1, "two", true]`).
    Array(Vec<JsonValue>),
    /// A JSON object mapping string keys to values (e.g. `{"key": "value"}`).
    Object(HashMap<String, JsonValue>),
}

trait JsonFormat {
    fn to_json_string(&self) -> String;
}

impl JsonFormat for f64 {
    fn to_json_string(&self) -> String {
        if self.trunc() == *self {
            format!("{}", self.trunc())
        } else {
            format!("{}", self)
        }
    }
}

impl JsonFormat for String {
    fn to_json_string(&self) -> String {
        format!("\"{}\"", escape_json_string(self))
    }
}

impl JsonFormat for HashMap<String, JsonValue> {
    fn to_json_string(&self) -> String {
        let mut array_as_string = r#"{"#.to_string();

        for (index, (key, value)) in self.iter().enumerate() {
            if index > 0 {
                array_as_string.push(','); // Add comma before all but the first
            }

            let value_as_string = match value {
                JsonValue::Null => "null".to_string(),
                JsonValue::Boolean(b) => b.to_string(),
                JsonValue::Number(n) => n.to_json_string(),
                JsonValue::String(s) => s.to_json_string(),
                JsonValue::Array(inner_array) => inner_array.to_json_string(),
                JsonValue::Object(inner_object) => inner_object.to_json_string(),
            };
            let item_as_string = format!("\"{}\": {}", key, value_as_string);
            array_as_string.push_str(&item_as_string);
        }
        array_as_string.push('}');
        array_as_string
    }
}

impl JsonFormat for [JsonValue] {
    fn to_json_string(&self) -> String {
        let mut array_as_string = r#"["#.to_string();

        for (index, item) in self.iter().enumerate() {
            if index > 0 {
                array_as_string.push(','); // Add comma before all but the first
            }

            let item_as_string = match item {
                JsonValue::Null => "null".to_string(),
                JsonValue::Boolean(b) => b.to_string(),
                JsonValue::Number(n) => n.to_json_string(),
                JsonValue::String(s) => s.to_json_string(),
                JsonValue::Array(inner_array) => inner_array.to_json_string(),
                JsonValue::Object(inner_object) => inner_object.to_json_string(),
            };
            array_as_string.push_str(&item_as_string);
        }
        array_as_string.push(']');
        array_as_string
    }
}

impl JsonValue {
    /// Returns `true` if this value is `JsonValue::Null`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_json_parser::parse_json;
    ///
    /// let value = parse_json("null")?;
    /// assert!(value.is_null());
    ///
    /// let value = parse_json("42")?;
    /// assert!(!value.is_null());
    /// # Ok::<(), rust_json_parser::JsonError>(())
    /// ```
    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }

    /// Returns the inner string slice if this is a `JsonValue::String`, or `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_json_parser::parse_json;
    ///
    /// let value = parse_json(r#""hello""#)?;
    /// assert_eq!(value.as_str(), Some("hello"));
    ///
    /// let value = parse_json("42")?;
    /// assert_eq!(value.as_str(), None);
    /// # Ok::<(), rust_json_parser::JsonError>(())
    /// ```
    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Returns the inner `f64` if this is a `JsonValue::Number`, or `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_json_parser::parse_json;
    ///
    /// let value = parse_json("3.14")?;
    /// assert_eq!(value.as_f64(), Some(3.14));
    ///
    /// let value = parse_json("true")?;
    /// assert_eq!(value.as_f64(), None);
    /// # Ok::<(), rust_json_parser::JsonError>(())
    /// ```
    pub fn as_f64(&self) -> Option<f64> {
        let JsonValue::Number(n) = self else {
            return None;
        };
        Some(*n)
    }

    /// Returns the inner `bool` if this is a `JsonValue::Boolean`, or `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_json_parser::parse_json;
    ///
    /// let value = parse_json("true")?;
    /// assert_eq!(value.as_bool(), Some(true));
    ///
    /// let value = parse_json("42")?;
    /// assert_eq!(value.as_bool(), None);
    /// # Ok::<(), rust_json_parser::JsonError>(())
    /// ```
    pub fn as_bool(&self) -> Option<bool> {
        let JsonValue::Boolean(b) = self else {
            return None;
        };
        Some(*b)
    }

    /// Returns a reference to the inner `Vec` if this is a `JsonValue::Array`, or `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_json_parser::parse_json;
    ///
    /// let value = parse_json("[1, 2, 3]")?;
    /// assert_eq!(value.as_array().map(|a| a.len()), Some(3));
    ///
    /// let value = parse_json("42")?;
    /// assert_eq!(value.as_array(), None);
    /// # Ok::<(), rust_json_parser::JsonError>(())
    /// ```
    pub fn as_array(&self) -> Option<&Vec<JsonValue>> {
        match self {
            JsonValue::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Returns a reference to the inner `HashMap` if this is a `JsonValue::Object`, or `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_json_parser::parse_json;
    ///
    /// let value = parse_json(r#"{"key": "value"}"#)?;
    /// assert_eq!(value.as_object().map(|o| o.len()), Some(1));
    ///
    /// let value = parse_json("[1, 2]")?;
    /// assert_eq!(value.as_object(), None);
    /// # Ok::<(), rust_json_parser::JsonError>(())
    /// ```
    pub fn as_object(&self) -> Option<&HashMap<String, JsonValue>> {
        match self {
            JsonValue::Object(o) => Some(o),
            _ => None,
        }
    }

    /// Looks up a value by key if this is a `JsonValue::Object`. Returns `None` if the
    /// key is missing or if this value is not an object.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_json_parser::{parse_json, JsonValue};
    ///
    /// let value = parse_json(r#"{"name": "Alice", "age": 30}"#)?;
    /// assert_eq!(value.get("name"), Some(&JsonValue::String("Alice".to_string())));
    /// assert_eq!(value.get("missing"), None);
    /// # Ok::<(), rust_json_parser::JsonError>(())
    /// ```
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        let object = self.as_object();
        match object {
            Some(o) => o.get(key),
            None => None,
        }
    }

    /// Looks up a value by index if this is a `JsonValue::Array`. Returns `None` if the
    /// index is out of bounds or if this value is not an array.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_json_parser::{parse_json, JsonValue};
    ///
    /// let value = parse_json("[10, 20, 30]")?;
    /// assert_eq!(value.get_index(1), Some(&JsonValue::Number(20.0)));
    /// assert_eq!(value.get_index(5), None);
    /// # Ok::<(), rust_json_parser::JsonError>(())
    /// ```
    pub fn get_index(&self, index: usize) -> Option<&JsonValue> {
        let array = self.as_array();
        match array {
            Some(a) => a.get(index),
            None => None,
        }
    }

    /// Serializes this value to a pretty-printed JSON string with the given number
    /// of spaces per indentation level.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_json_parser::parse_json;
    ///
    /// let value = parse_json(r#"{"key": [1, 2]}"#)?;
    /// let pretty = value.pretty_print(2);
    /// assert!(pretty.contains("\"key\""));
    /// assert!(pretty.contains('\n'));
    /// # Ok::<(), rust_json_parser::JsonError>(())
    /// ```
    pub fn pretty_print(&self, indent: usize) -> String {
        self.pretty_print_recursive(0, indent)
    }

    /// Recursive helper for [`pretty_print`](Self::pretty_print) that tracks the current
    /// nesting depth.
    fn pretty_print_recursive(&self, depth: usize, indent: usize) -> String {
        let pad = " ".repeat(depth * indent);
        let inner_pad = " ".repeat((depth + 1) * indent);

        match self {
            JsonValue::Null => "null".to_string(),
            JsonValue::Boolean(b) => b.to_string(),
            JsonValue::Number(n) => n.to_json_string(),
            JsonValue::String(s) => s.to_json_string(),
            JsonValue::Array(arr) => {
                if arr.is_empty() {
                    return "[]".to_string();
                }
                let items: Vec<String> = arr
                    .iter()
                    .map(|v| {
                        format!(
                            "{}{}",
                            inner_pad,
                            v.pretty_print_recursive(depth + 1, indent)
                        )
                    })
                    .collect();
                format!("[\n{}\n{}]", items.join(",\n"), pad)
            }
            JsonValue::Object(obj) => {
                if obj.is_empty() {
                    return "{}".to_string();
                }
                let entries: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| {
                        format!(
                            "{}\"{}\": {}",
                            inner_pad,
                            escape_json_string(k),
                            v.pretty_print_recursive(depth + 1, indent)
                        )
                    })
                    .collect();
                format!("{{\n{}\n{}}}", entries.join(",\n"), pad)
            }
        }
    }
}

impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonValue::Null => write!(f, "null"),
            JsonValue::Boolean(b) => write!(f, "{}", b),
            JsonValue::Number(n) => write!(f, "{}", n.to_json_string()),
            JsonValue::String(s) => write!(f, "{}", s.to_json_string()),
            JsonValue::Array(array) => write!(f, "{}", array.to_json_string()),
            JsonValue::Object(object) => write!(f, "{}", object.to_json_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_value_creation() {
        let null_val = JsonValue::Null;
        let bool_val = JsonValue::Boolean(true);
        let num_val = JsonValue::Number(42.5);
        let str_val = JsonValue::String("hello".to_string());

        assert!(null_val.is_null());
        assert_eq!(bool_val.as_bool(), Some(true));
        assert_eq!(num_val.as_f64(), Some(42.5));
        assert_eq!(str_val.as_str(), Some("hello"));
    }

    #[test]
    fn test_json_value_accessors() {
        let value = JsonValue::String("test".to_string());
        assert_eq!(value.as_str(), Some("test"));
        assert_eq!(value.as_f64(), None);
        assert_eq!(value.as_bool(), None);
        assert!(!value.is_null());

        let value = JsonValue::Number(42.0);
        assert_eq!(value.as_f64(), Some(42.0));
        assert_eq!(value.as_str(), None);

        let value = JsonValue::Boolean(true);
        assert_eq!(value.as_bool(), Some(true));

        let value = JsonValue::Null;
        assert!(value.is_null());
    }

    #[test]
    fn test_json_value_equality() {
        assert_eq!(JsonValue::Null, JsonValue::Null);
        assert_eq!(JsonValue::Boolean(true), JsonValue::Boolean(true));
        assert_eq!(JsonValue::Number(42.0), JsonValue::Number(42.0));
        assert_eq!(
            JsonValue::String("test".to_string()),
            JsonValue::String("test".to_string())
        );

        assert_ne!(JsonValue::Null, JsonValue::Boolean(false));
        assert_ne!(JsonValue::Number(1.0), JsonValue::Number(2.0));
    }

    #[test]
    fn test_display_primitives() {
        assert_eq!(JsonValue::Null.to_string(), "null");
        assert_eq!(JsonValue::Boolean(true).to_string(), "true");
        assert_eq!(JsonValue::Boolean(false).to_string(), "false");
        assert_eq!(JsonValue::Number(42.0).to_string(), "42");
        assert_eq!(JsonValue::Number(3.14).to_string(), "3.14");
        assert_eq!(
            JsonValue::String("hello".to_string()).to_string(),
            "\"hello\""
        );
    }

    #[test]
    fn test_display_array() {
        let value = JsonValue::Array(vec![JsonValue::Number(1.0), JsonValue::Number(2.0)]);
        assert_eq!(value.to_string(), "[1,2]");
    }

    #[test]
    fn test_display_empty_containers() {
        assert_eq!(JsonValue::Array(vec![]).to_string(), "[]");
        assert_eq!(JsonValue::Object(HashMap::new()).to_string(), "{}");
    }

    #[test]
    fn test_display_escape_string() {
        let value = JsonValue::String("hello\nworld".to_string());
        assert_eq!(value.to_string(), "\"hello\\nworld\"");
    }

    #[test]
    fn test_display_escape_quotes() {
        let value = JsonValue::String("say \"hi\"".to_string());
        assert_eq!(value.to_string(), "\"say \\\"hi\\\"\"");
    }
}
