pub mod error;
pub mod parser;
pub mod tokenizer;
pub mod value;

// Re-export types - make them accessible from the top level
// Without this: users write `use my_lib::parser::parse_json`
// With this: users write `use my_lib::parse_json` (cleaner!)
pub use error::JsonError;
pub use parser::parse_json;
pub use tokenizer::{Token, Tokenizer};
pub use value::JsonValue;

// Type alias for convenience
// Users can write Result<JsonValue> instead of std::result::Result<JsonValue, JsonError>
pub type Result<T> = std::result::Result<T, JsonError>;

// Copy these tests as-is:
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration() {
        // Test the full parsing pipeline
        assert_eq!(parse_json("42").unwrap(), JsonValue::Number(42.0));
        assert_eq!(parse_json("true").unwrap(), JsonValue::Boolean(true));
        assert_eq!(parse_json("null").unwrap(), JsonValue::Null);
        assert_eq!(
            parse_json(r#""hello""#).unwrap(),
            JsonValue::String("hello".to_string())
        );
    }

    #[test]
    fn test_error_propagation() {
        // Test that errors propagate properly with correct details
        let result = parse_json("@invalid@");
        assert!(result.is_err());

        // Validate error details through pattern matching
        match result {
            Err(JsonError::UnexpectedToken {
                expected,
                found,
                position,
            }) => {
                assert_eq!(expected, "valid JSON token");
                assert_eq!(found, "@");
                assert_eq!(position, 0);
            }
            _ => panic!("Expected UnexpectedToken error"),
        }
    }
}
