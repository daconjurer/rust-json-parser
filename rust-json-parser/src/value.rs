use crate::utils::escape_json_string;
use std::{collections::HashMap, fmt};

/*
 * Enum for JsonValue kind. Valid variants:
 * String(String), Number(f64), Boolean(bool), Null
 */
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Array(Vec<JsonValue>),              // A JSON array is a Vec of values
    Object(HashMap<String, JsonValue>), // A JSON object is a HashMap
}

fn number_to_string(input: &f64) -> String {
    match input.trunc() == *input {
        true => format!("{}", input.trunc()),
        false => format!("{}", input),
    }
}

fn from_json_value(input: &str) -> String {
    format!("\"{}\"", escape_json_string(input))
}

fn object_to_string(input: &HashMap<String, JsonValue>) -> String {
    let mut array_as_string = r#"{"#.to_string();

    for (index, (key, value)) in input.iter().enumerate() {
        if index > 0 {
            array_as_string.push(','); // Add comma before all but the first
        }

        let value_as_string = match value {
            JsonValue::Null => "null".to_string(),
            JsonValue::Boolean(b) => b.to_string(),
            JsonValue::Number(n) => number_to_string(n),
            JsonValue::String(s) => from_json_value(s),
            JsonValue::Array(array) => array_to_string(array),
            JsonValue::Object(object) => object_to_string(object),
        };
        let item_as_string = format!("\"{}\": {}", key, value_as_string);
        array_as_string.push_str(&item_as_string);
    }
    array_as_string.push('}');
    array_as_string
}

fn array_to_string(input: &[JsonValue]) -> String {
    let mut array_as_string = r#"["#.to_string();

    for (index, item) in input.iter().enumerate() {
        if index > 0 {
            array_as_string.push(','); // Add comma before all but the first
        }

        let item_as_string = match item {
            JsonValue::Null => "null".to_string(),
            JsonValue::Boolean(b) => b.to_string(),
            JsonValue::Number(n) => number_to_string(n),
            JsonValue::String(s) => from_json_value(s),
            JsonValue::Array(array) => array_to_string(array),
            JsonValue::Object(object) => object_to_string(object),
        };
        array_as_string.push_str(&item_as_string);
    }
    array_as_string.push(']');
    array_as_string
}

impl JsonValue {
    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        let JsonValue::Number(n) = self else {
            return None;
        };
        Some(*n)
    }

    pub fn as_bool(&self) -> Option<bool> {
        let JsonValue::Boolean(b) = self else {
            return None;
        };
        Some(*b)
    }

    pub fn as_array(&self) -> Option<&Vec<JsonValue>> {
        match self {
            JsonValue::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, JsonValue>> {
        match self {
            JsonValue::Object(o) => Some(o),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        let object = self.as_object();
        match object {
            Some(o) => o.get(key),
            None => None,
        }
    }

    pub fn get_index(&self, index: usize) -> Option<&JsonValue> {
        let array = self.as_array();
        match array {
            Some(a) => a.get(index),
            None => None,
        }
    }
}

impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonValue::Null => write!(f, "null"),
            JsonValue::Boolean(b) => write!(f, "{}", b),
            JsonValue::Number(n) => write!(f, "{}", number_to_string(n)),
            JsonValue::String(s) => write!(f, "{}", from_json_value(s)),
            JsonValue::Array(array) => write!(f, "{}", array_to_string(array)),
            JsonValue::Object(object) => write!(f, "{}", object_to_string(object)),
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
