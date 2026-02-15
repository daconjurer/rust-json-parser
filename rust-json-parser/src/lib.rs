pub mod error;
pub mod parser;
pub mod tokenizer;
pub mod utils;
pub mod value;

// Re-export types - make them accessible from the top level
// Without this: users write `use my_lib::parser::parse_json`
// With this: users write `use my_lib::parse_json` (cleaner!)
pub use error::JsonError;
pub use parser::JsonParser;
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
        let mut parser = JsonParser::new("42").unwrap();
        assert_eq!(parser.parse().unwrap(), JsonValue::Number(42.0));

        let mut parser = JsonParser::new("true").unwrap();
        assert_eq!(parser.parse().unwrap(), JsonValue::Boolean(true));

        let mut parser = JsonParser::new("null").unwrap();
        assert_eq!(parser.parse().unwrap(), JsonValue::Null);

        let mut parser = JsonParser::new(r#""hello""#).unwrap();
        assert_eq!(
            parser.parse().unwrap(),
            JsonValue::String("hello".to_string())
        );
    }

    #[test]
    fn test_error_propagation() {
        // Test that errors propagate properly with correct details
        let result = JsonParser::new("@invalid@");
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
