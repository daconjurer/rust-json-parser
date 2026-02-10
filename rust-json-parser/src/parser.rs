use crate::Result;
use crate::error::JsonError;
use crate::tokenizer::{Token, Tokenizer};
use crate::value::JsonValue;

pub fn parse_json(input: &str) -> Result<JsonValue> {
    // 1. Call tokenize(input)?  (? propagates errors)
    let tokens = Tokenizer::new(input).tokenize()?;

    // 2. Check if tokens is empty
    let token = tokens.first().ok_or(JsonError::UnexpectedEndOfInput {
        expected: "JSON value".to_string(),
        position: 0,
    })?;

    // 3. Match on token and convert to JsonValue
    match token {
        Token::String(s) => Ok(JsonValue::String(s.clone())),
        Token::Number(n) => Ok(JsonValue::Number(*n)),
        Token::Boolean(b) => Ok(JsonValue::Boolean(*b)),
        Token::Null => Ok(JsonValue::Null),
        _ => Err(JsonError::UnexpectedToken {
            expected: "string".to_string(),
            found: format!("{:?}", token),
            position: 0,
        }),
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_parse_string() {
//         let result = parse_json(r#""hello world""#).unwrap();
//         assert_eq!(result, JsonValue::String("hello world".to_string()));
//     }

//     #[test]
//     fn test_parse_number() {
//         let result = parse_json("42.5").unwrap();
//         assert_eq!(result, JsonValue::Number(42.5));

//         let result = parse_json("0").unwrap();
//         assert_eq!(result, JsonValue::Number(0.0));

//         let result = parse_json("-10").unwrap();
//         assert_eq!(result, JsonValue::Number(-10.0));
//     }

//     #[test]
//     fn test_parse_boolean() {
//         let result = parse_json("true").unwrap();
//         assert_eq!(result, JsonValue::Boolean(true));

//         let result = parse_json("false").unwrap();
//         assert_eq!(result, JsonValue::Boolean(false));
//     }

//     #[test]
//     fn test_parse_null() {
//         let result = parse_json("null").unwrap();
//         assert_eq!(result, JsonValue::Null);
//     }

//     #[test]
//     fn test_parse_error_empty() {
//         let result = parse_json("");
//         assert!(result.is_err());

//         match result {
//             Err(JsonError::UnexpectedEndOfInput { expected, position }) => {
//                 assert_eq!(expected, "JSON value");
//                 assert_eq!(position, 0);
//             }
//             _ => panic!("Expected UnexpectedEndOfInput error"),
//         }
//     }

//     #[test]
//     fn test_parse_error_invalid_token() {
//         let result = parse_json("@");
//         assert!(result.is_err());
//     }

//     #[test]
//     fn test_parse_with_whitespace() {
//         let result = parse_json("  42  ").unwrap();
//         assert_eq!(result, JsonValue::Number(42.0));

//         let result = parse_json("\n\ttrue\n").unwrap();
//         assert_eq!(result, JsonValue::Boolean(true));
//     }

//     #[test]
//     fn test_result_pattern_matching() {
//         let result = parse_json("42");

//         match result {
//             Ok(JsonValue::Number(n)) => assert_eq!(n, 42.0),
//             _ => panic!("Expected successful number parse"),
//         }

//         let result = parse_json("@invalid@");

//         match result {
//             Err(JsonError::UnexpectedToken { .. }) => {} // Expected
//             _ => panic!("Expected UnexpectedToken error"),
//         }
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     // === Struct Usage Tests ===

//     #[test]
//     fn test_parser_creation() {
//         let parser = JsonParser::new("42");
//         assert!(parser.is_ok());
//     }

//     #[test]
//     fn test_parser_creation_tokenize_error() {
//         let parser = JsonParser::new(r#""\q""#); // Invalid escape
//         assert!(parser.is_err());
//     }

//     // === Primitive Parsing Tests ===

//     #[test]
//     fn test_parse_number() {
//         let mut parser = JsonParser::new("42").unwrap();
//         let value = parser.parse().unwrap();
//         assert_eq!(value, JsonValue::Number(42.0));
//     }

//     #[test]
//     fn test_parse_negative_number() {
//         let mut parser = JsonParser::new("-3.14").unwrap();
//         let value = parser.parse().unwrap();
//         assert_eq!(value, JsonValue::Number(-3.14));
//     }

//     #[test]
//     fn test_parse_boolean_true() {
//         let mut parser = JsonParser::new("true").unwrap();
//         let value = parser.parse().unwrap();
//         assert_eq!(value, JsonValue::Boolean(true));
//     }

//     #[test]
//     fn test_parse_boolean_false() {
//         let mut parser = JsonParser::new("false").unwrap();
//         let value = parser.parse().unwrap();
//         assert_eq!(value, JsonValue::Boolean(false));
//     }

//     #[test]
//     fn test_parse_null() {
//         let mut parser = JsonParser::new("null").unwrap();
//         let value = parser.parse().unwrap();
//         assert_eq!(value, JsonValue::Null);
//     }

//     #[test]
//     fn test_parse_simple_string() {
//         let mut parser = JsonParser::new(r#""hello""#).unwrap();
//         let value = parser.parse().unwrap();
//         assert_eq!(value, JsonValue::String("hello".to_string()));
//     }

//     // === Escape Sequence Integration Tests ===

//     #[test]
//     fn test_parse_string_with_newline() {
//         let mut parser = JsonParser::new(r#""hello\nworld""#).unwrap();
//         let value = parser.parse().unwrap();
//         assert_eq!(value, JsonValue::String("hello\nworld".to_string()));
//     }

//     #[test]
//     fn test_parse_string_with_tab() {
//         let mut parser = JsonParser::new(r#""col1\tcol2""#).unwrap();
//         let value = parser.parse().unwrap();
//         assert_eq!(value, JsonValue::String("col1\tcol2".to_string()));
//     }

//     #[test]
//     fn test_parse_string_with_quotes() {
//         let mut parser = JsonParser::new(r#""say \"hi\"""#).unwrap();
//         let value = parser.parse().unwrap();
//         assert_eq!(value, JsonValue::String("say \"hi\"".to_string()));
//     }

//     #[test]
//     fn test_parse_string_with_unicode() {
//         let mut parser = JsonParser::new(r#""\u0048\u0065\u006c\u006c\u006f""#).unwrap();
//         let value = parser.parse().unwrap();
//         assert_eq!(value, JsonValue::String("Hello".to_string()));
//     }

//     #[test]
//     fn test_parse_complex_escapes() {
//         let mut parser = JsonParser::new(r#""line1\nline2\t\"quoted\"\u0021""#).unwrap();
//         let value = parser.parse().unwrap();
//         assert_eq!(
//             value,
//             JsonValue::String("line1\nline2\t\"quoted\"!".to_string())
//         );
//     }

//     // === Error Tests ===

//     #[test]
//     fn test_parse_empty_input() {
//         let parser = JsonParser::new("");
//         // Could fail at tokenization (no tokens) or parsing (empty token list)
//         // Either is acceptable - just verify it's an error
//         assert!(parser.is_err() || parser.unwrap().parse().is_err());
//     }

//     #[test]
//     fn test_parse_whitespace_only() {
//         let parser = JsonParser::new("   ");
//         assert!(parser.is_err() || parser.unwrap().parse().is_err());
//     }
// }
