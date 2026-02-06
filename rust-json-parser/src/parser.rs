use crate::error::JsonError;
use crate::tokenizer::{Token, tokenize};
use crate::value::JsonValue;

type Result<T> = std::result::Result<T, JsonError>;

pub fn parse_json(input: &str) -> Result<JsonValue> {
    // 1. Call tokenize(input)?  (? propagates errors)
    let mut _parsed_values: Vec<JsonValue> = Vec::new();
    let tokens = tokenize(input)?;

    // 2. Check if tokens is empty
    let token = tokens.first().ok_or(JsonError::UnexpectedEndOfInput {
        expected: "JSON value".to_string(),
        position: 0,
    })?;

    // 3. Match on token and convert to JsonValue
    match token {
        Token::String(s) => Ok(JsonValue::String(s.clone())),
        _ => Err(JsonError::UnexpectedToken {
            expected: "string".to_string(),
            found: format!("{:?}", token),
            position: 0,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string() {
        let result = parse_json(r#""hello world""#).unwrap();
        assert_eq!(result, JsonValue::String("hello world".to_string()));
    }

    // #[test]
    // fn test_parse_number() {
    //     let result = parse_json("42.5").unwrap();
    //     assert_eq!(result, JsonValue::Number(42.5));

    //     let result = parse_json("0").unwrap();
    //     assert_eq!(result, JsonValue::Number(0.0));

    //     let result = parse_json("-10").unwrap();
    //     assert_eq!(result, JsonValue::Number(-10.0));
    // }

    // #[test]
    // fn test_parse_boolean() {
    //     let result = parse_json("true").unwrap();
    //     assert_eq!(result, JsonValue::Boolean(true));

    //     let result = parse_json("false").unwrap();
    //     assert_eq!(result, JsonValue::Boolean(false));
    // }

    // #[test]
    // fn test_parse_null() {
    //     let result = parse_json("null").unwrap();
    //     assert_eq!(result, JsonValue::Null);
    // }

    #[test]
    fn test_parse_error_empty() {
        let result = parse_json("");
        assert!(result.is_err());

        match result {
            Err(JsonError::UnexpectedEndOfInput { expected, position }) => {
                assert_eq!(expected, "JSON value");
                assert_eq!(position, 0);
            }
            _ => panic!("Expected UnexpectedEndOfInput error"),
        }
    }

    #[test]
    fn test_parse_error_invalid_token() {
        let result = parse_json("@");
        assert!(result.is_err());
    }

    // #[test]
    // fn test_parse_with_whitespace() {
    //     let result = parse_json("  42  ").unwrap();
    //     assert_eq!(result, JsonValue::Number(42.0));

    //     let result = parse_json("\n\ttrue\n").unwrap();
    //     assert_eq!(result, JsonValue::Boolean(true));
    // }

    // #[test]
    // fn test_result_pattern_matching() {
    //     let result = parse_json("42");

    //     match result {
    //         Ok(JsonValue::Number(n)) => assert_eq!(n, 42.0),
    //         _ => panic!("Expected successful number parse"),
    //     }

    //     let result = parse_json("@invalid@");

    //     match result {
    //         Err(JsonError::UnexpectedToken { .. }) => {}, // Expected
    //         _ => panic!("Expected UnexpectedToken error"),
    //     }
    // }
}
