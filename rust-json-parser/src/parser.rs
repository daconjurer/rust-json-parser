use std::collections::HashMap;

use crate::Result;
use crate::error::JsonError;
use crate::tokenizer::{Token, Tokenizer};
use crate::value::JsonValue;

pub struct JsonParser {
    pub tokens: Vec<Token>, // TODO make me private
    #[allow(unused)]
    current: usize,
}

impl JsonParser {
    pub fn new(input: &str) -> Result<Self> {
        let mut tokenizer = Tokenizer::new(input);
        let tokens = tokenizer.tokenize()?;
        Ok(Self { current: 0, tokens })
    }

    pub fn parse(&mut self) -> Result<JsonValue> {
        match self.peek() {
            Some(Token::LeftBrace) => self.parse_object(),
            Some(Token::LeftBracket) => {
                self.advance(); // Consume opening [
                self.parse_array()
            }
            Some(_) => self.parse_primitive(),
            None => Err(JsonError::UnexpectedEndOfInput {
                expected: "string".to_string(),
                position: self.current,
            }),
        }
    }

    fn parse_primitive(&mut self) -> Result<JsonValue> {
        match self.peek() {
            Some(Token::String(s)) => Ok(JsonValue::String(s.clone())),
            Some(Token::Number(n)) => Ok(JsonValue::Number(*n)),
            Some(Token::Boolean(b)) => Ok(JsonValue::Boolean(*b)),
            Some(Token::Null) => Ok(JsonValue::Null),
            Some(_) => Err(JsonError::UnexpectedToken {
                expected: "string".to_string(),
                found: "TODO token to_string()".to_string(),
                position: self.current,
            }),
            None => Err(JsonError::UnexpectedEndOfInput {
                expected: "string".to_string(),
                position: self.current,
            }),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue> {
        let mut array = Vec::new();
        let mut expect_comma = false;

        while let Some(token) = self.peek() {
            match token {
                // Start of array
                Token::LeftBracket => {
                    if expect_comma {
                        return Err(JsonError::UnexpectedToken {
                            expected: ",".to_string(),
                            found: format!("{:?}", token),
                            position: self.current,
                        });
                    }

                    self.advance(); // Consume opening [
                    let nested_array = self.parse_array()?;
                    array.push(nested_array);
                    expect_comma = true;
                }
                // End of array
                Token::RightBracket => {
                    self.advance(); // Consume closig ]
                    return Ok(JsonValue::Array(array));
                }
                // Start of object (opening { is consumed by parse_object())
                Token::LeftBrace => {
                    if expect_comma {
                        return Err(JsonError::UnexpectedToken {
                            expected: ",".to_string(),
                            found: format!("{:?}", token),
                            position: self.current,
                        });
                    }

                    let nested_object = self.parse_object()?;
                    array.push(nested_object);
                    expect_comma = true;
                }
                Token::String(s) => {
                    if expect_comma {
                        return Err(JsonError::UnexpectedToken {
                            expected: ",".to_string(),
                            found: format!("{:?}", token),
                            position: self.current,
                        });
                    }

                    array.push(JsonValue::String(s.clone()));
                    self.advance();
                    expect_comma = true;
                }
                Token::Number(n) => {
                    if expect_comma {
                        return Err(JsonError::UnexpectedToken {
                            expected: ",".to_string(),
                            found: format!("{:?}", token),
                            position: self.current,
                        });
                    }

                    array.push(JsonValue::Number(*n));
                    self.advance();
                    expect_comma = true;
                }
                Token::Boolean(b) => {
                    if expect_comma {
                        return Err(JsonError::UnexpectedToken {
                            expected: ",".to_string(),
                            found: format!("{:?}", token),
                            position: self.current,
                        });
                    }

                    array.push(JsonValue::Boolean(*b));
                    self.advance();
                    expect_comma = true;
                }
                Token::Null => {
                    if expect_comma {
                        return Err(JsonError::UnexpectedToken {
                            expected: ",".to_string(),
                            found: format!("{:?}", token),
                            position: self.current,
                        });
                    }

                    array.push(JsonValue::Null);
                    self.advance();
                    expect_comma = true;
                }
                Token::Comma => {
                    self.advance(); // Consume comma
                    if !expect_comma {
                        return Err(JsonError::UnexpectedToken {
                            expected: "closing bracket".to_string(),
                            found: ",".to_string(),
                            position: self.current,
                        });
                    }
                    if matches!(self.peek(), Some(Token::RightBracket)) {
                        return Err(JsonError::UnexpectedToken {
                            expected: "string, bool, number or object".to_string(),
                            found: "}".to_string(),
                            position: self.current,
                        });
                    }
                    expect_comma = false;
                }
                _ => {
                    return Err(JsonError::UnexpectedToken {
                        expected: "valid JSON value".to_string(),
                        found: format!("{:?}", token),
                        position: self.current,
                    });
                }
            };
        }

        Err(JsonError::UnexpectedEndOfInput {
            expected: "closing bracket".to_string(),
            position: self.current,
        })
    }

    fn parse_object(&mut self) -> Result<JsonValue> {
        let mut key = String::new();
        let mut object = HashMap::new();
        let mut colon_found = false;
        let mut expect_comma = false;

        while let Some(token) = self.peek() {
            match token {
                // Start of object
                Token::LeftBrace => {
                    if expect_comma {
                        return Err(JsonError::UnexpectedToken {
                            expected: ",".to_string(),
                            found: format!("{:?}", token),
                            position: self.current,
                        });
                    }

                    self.advance(); // Consume opening {
                    if colon_found {
                        let nested_object = self.parse_object()?;
                        object.insert(key.clone(), nested_object);
                        colon_found = false;
                        expect_comma = true;
                    }
                }
                // End of object
                Token::RightBrace => {
                    self.advance(); // Consume closing }
                    return Ok(JsonValue::Object(object));
                }
                // Start of array (end of array is handled in parse_array())
                Token::LeftBracket => {
                    if expect_comma {
                        return Err(JsonError::UnexpectedToken {
                            expected: ",".to_string(),
                            found: format!("{:?}", token),
                            position: self.current,
                        });
                    }

                    if colon_found {
                        self.advance(); // Consume opening [
                        let array = self.parse_array()?;
                        object.insert(key.clone(), array);
                        colon_found = false;
                        expect_comma = true;
                    }
                }
                // Key or string value
                Token::String(s) => {
                    if expect_comma {
                        return Err(JsonError::UnexpectedToken {
                            expected: ",".to_string(),
                            found: format!("{:?}", token),
                            position: self.current,
                        });
                    }

                    // Key
                    if !colon_found {
                        // Check if next token is a colon
                        if self.get_token(self.current + 1) != Some(&Token::Colon) {
                            return Err(JsonError::UnexpectedToken {
                                expected: ":".to_string(),
                                found: "TODO token to_string()".to_string(),
                                position: self.current,
                            });
                        }
                        key = s.clone();
                    // Value
                    } else {
                        object.insert(key.clone(), JsonValue::String(s.clone()));
                        colon_found = false;
                        expect_comma = true;
                    }
                    self.advance();
                }
                Token::Number(n) => {
                    if expect_comma {
                        return Err(JsonError::UnexpectedToken {
                            expected: ",".to_string(),
                            found: format!("{:?}", token),
                            position: self.current,
                        });
                    }

                    if !colon_found {
                        return Err(JsonError::UnexpectedToken {
                            expected: "string".to_string(),
                            found: n.to_string(),
                            position: self.current,
                        });
                    } else {
                        object.insert(key.clone(), JsonValue::Number(*n));
                        colon_found = false;
                        expect_comma = true;
                    }
                    self.advance();
                }
                Token::Boolean(b) => {
                    if expect_comma {
                        return Err(JsonError::UnexpectedToken {
                            expected: ",".to_string(),
                            found: format!("{:?}", token),
                            position: self.current,
                        });
                    }

                    if !colon_found {
                        return Err(JsonError::UnexpectedToken {
                            expected: "string".to_string(),
                            found: b.to_string(),
                            position: self.current,
                        });
                    } else {
                        object.insert(key.clone(), JsonValue::Boolean(*b));
                        colon_found = false;
                        expect_comma = true;
                    }
                    self.advance();
                }
                Token::Null => {
                    if expect_comma {
                        return Err(JsonError::UnexpectedToken {
                            expected: ",".to_string(),
                            found: format!("{:?}", token),
                            position: self.current,
                        });
                    }

                    if !colon_found {
                        return Err(JsonError::UnexpectedToken {
                            expected: "string".to_string(),
                            found: "null".to_string(),
                            position: self.current,
                        });
                    } else {
                        object.insert(key.clone(), JsonValue::Null);
                        colon_found = false;
                        expect_comma = true;
                    }
                    self.advance();
                }
                Token::Colon => {
                    colon_found = true;
                    self.advance();
                }
                Token::Comma => {
                    self.advance(); // Consume comma
                    if !expect_comma {
                        return Err(JsonError::UnexpectedToken {
                            expected: "closing brace".to_string(),
                            found: ",".to_string(),
                            position: self.current,
                        });
                    }
                    if matches!(self.peek(), Some(Token::RightBrace)) {
                        return Err(JsonError::UnexpectedToken {
                            expected: "string".to_string(),
                            found: "}".to_string(),
                            position: self.current,
                        });
                    }
                    expect_comma = false;
                }
                _ => {
                    return Err(JsonError::UnexpectedToken {
                        expected: "valid JSON value".to_string(),
                        found: format!("{:?}", token),
                        position: self.current,
                    });
                }
            };
        }

        Err(JsonError::UnexpectedEndOfInput {
            expected: "closing brace".to_string(),
            position: self.current,
        })
    }

    /*
     * Look at current token without advancing
     */
    fn peek(&self) -> Option<&Token> {
        if !self.is_at_end() {
            return self.tokens.get(self.current);
        }
        None
    }

    fn get_token(&self, index: usize) -> Option<&Token> {
        self.tokens.get(index)
    }

    /*
     * Move forward, return previous token
     */
    fn advance(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.current).cloned();
        self.current += 1;
        token
    }

    /*
     * Check if the input has been consumed
     */
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }
}

pub fn parse_json(input: &str) -> Result<JsonValue> {
    JsonParser::new(input)?.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Struct Usage Tests ===

    #[test]
    fn test_parser_creation() {
        let parser = JsonParser::new("42");
        assert!(parser.is_ok());
    }

    #[test]
    fn test_parser_creation_tokenize_invalid_escape_error() {
        let parser = JsonParser::new(r#""\q""#); // Invalid escape
        assert!(matches!(parser, Err(JsonError::InvalidEscape { .. })));
    }

    #[test]
    fn test_parser_creation_tokenize_invalid_json_token_error() {
        let parser = JsonParser::new(r#"@"#); // Invalid JSON token
        assert!(matches!(parser, Err(JsonError::UnexpectedToken { .. })));
    }

    // === Primitive Parsing Tests ===

    #[test]
    fn test_parse_number() {
        let mut parser = JsonParser::new("42").unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(value, JsonValue::Number(42.0));
    }

    #[test]
    fn test_parse_negative_number() {
        let mut parser = JsonParser::new("-3.14").unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(value, JsonValue::Number(-3.14));
    }

    #[test]
    fn test_parse_boolean_true() {
        let mut parser = JsonParser::new("true").unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(value, JsonValue::Boolean(true));
    }

    #[test]
    fn test_parse_boolean_false() {
        let mut parser = JsonParser::new("false").unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(value, JsonValue::Boolean(false));
    }

    #[test]
    fn test_parse_null() {
        let mut parser = JsonParser::new("null").unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(value, JsonValue::Null);
    }

    #[test]
    fn test_parse_simple_string() {
        let mut parser = JsonParser::new(r#""hello""#).unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(value, JsonValue::String("hello".to_string()));
    }

    // === Escape Sequence Integration Tests ===

    #[test]
    fn test_parse_string_with_newline() {
        let mut parser = JsonParser::new(r#""hello\nworld""#).unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(value, JsonValue::String("hello\nworld".to_string()));
    }

    #[test]
    fn test_parse_string_with_tab() {
        let mut parser = JsonParser::new(r#""col1\tcol2""#).unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(value, JsonValue::String("col1\tcol2".to_string()));
    }

    #[test]
    fn test_parse_string_with_quotes() {
        let mut parser = JsonParser::new(r#""say \"hi\"""#).unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(value, JsonValue::String("say \"hi\"".to_string()));
    }

    #[test]
    fn test_parse_string_with_unicode() {
        let mut parser = JsonParser::new(r#""\u0048\u0065\u006c\u006c\u006f""#).unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(value, JsonValue::String("Hello".to_string()));
    }

    #[test]
    fn test_parse_complex_escapes() {
        let mut parser = JsonParser::new(r#""line1\nline2\t\"quoted\"\u0021""#).unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(
            value,
            JsonValue::String("line1\nline2\t\"quoted\"!".to_string())
        );
    }

    // === Error Tests ===

    #[test]
    fn test_parse_empty_input() {
        let parser = JsonParser::new("");
        // Could fail at tokenization (no tokens) or parsing (empty token list)
        // Either is acceptable - just verify it's an error
        assert!(parser.is_err() || parser.unwrap().parse().is_err());
    }

    #[test]
    fn test_parse_whitespace_only() {
        let parser = JsonParser::new("   ");
        assert!(parser.is_err() || parser.unwrap().parse().is_err());
    }

    #[test]
    fn test_error_unclosed_array() {
        let result = parse_json("[1, 2");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_unclosed_object() {
        let result = parse_json(r#"{"key": 1"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_trailing_comma_array() {
        let result = parse_json("[1, 2,]");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_trailing_comma_object() {
        let result = parse_json(r#"{"a": 1,}"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_missing_colon() {
        let result = parse_json(r#"{"key" 1}"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_invalid_key() {
        let result = parse_json(r#"{123: "value"}"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_missing_comma_array() {
        let result = parse_json("[1 2 3]");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_missing_comma_object() {
        let result = parse_json(r#"{"a": 1 "b": 2}"#);
        assert!(result.is_err());
    }

    // === Arrays Tests ===

    #[test]
    fn test_parse_empty_array() {
        let value = parse_json("[]").unwrap();
        assert_eq!(value, JsonValue::Array(vec![]));
    }

    #[test]
    fn test_parse_array_single() {
        let value = parse_json("[1]").unwrap();
        assert_eq!(value, JsonValue::Array(vec![JsonValue::Number(1.0)]));
    }

    #[test]
    fn test_parse_array_multiple() {
        let value = parse_json("[1, 2, 3]").unwrap();
        let expected = JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::Number(2.0),
            JsonValue::Number(3.0),
        ]);
        assert_eq!(value, expected);
    }

    #[test]
    fn test_parse_array_mixed_types() {
        let value = parse_json(r#"[1, "two", true, null]"#).unwrap();
        let expected = JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::String("two".to_string()),
            JsonValue::Boolean(true),
            JsonValue::Null,
        ]);
        assert_eq!(value, expected);
    }

    #[test]
    fn test_parse_nested_arrays() {
        let value = parse_json("[[1, 2], [3, 4]]").unwrap();
        let expected = JsonValue::Array(vec![
            JsonValue::Array(vec![JsonValue::Number(1.0), JsonValue::Number(2.0)]),
            JsonValue::Array(vec![JsonValue::Number(3.0), JsonValue::Number(4.0)]),
        ]);
        assert_eq!(value, expected);
    }

    #[test]
    fn test_parse_deeply_nested() {
        let value = parse_json("[[[1]]]").unwrap();
        let expected = JsonValue::Array(vec![JsonValue::Array(vec![JsonValue::Array(vec![
            JsonValue::Number(1.0),
        ])])]);
        assert_eq!(value, expected);
    }

    // #[test]
    // fn test_array_accessor() {
    //     let value = parse_json("[1, 2, 3]").unwrap();
    //     let arr = value.as_array().unwrap();
    //     assert_eq!(arr.len(), 3);
    // }

    // #[test]
    // fn test_array_get_index() {
    //     let value = parse_json("[10, 20, 30]").unwrap();
    //     assert_eq!(value.get_index(1), Some(&JsonValue::Number(20.0)));
    //     assert_eq!(value.get_index(5), None);
    // }

    // === Objects Tests ===

    #[test]
    fn test_parse_empty_object() {
        let value = parse_json("{}").unwrap();
        assert_eq!(value, JsonValue::Object(HashMap::new()));
    }

    #[test]
    fn test_parse_object_single_key() {
        let value = parse_json(r#"{"key": "value"}"#).unwrap();
        let mut expected = HashMap::new();
        expected.insert("key".to_string(), JsonValue::String("value".to_string()));
        assert_eq!(value, JsonValue::Object(expected));
    }

    #[test]
    fn test_parse_object_multiple_keys() {
        let value = parse_json(r#"{"name": "Alice", "age": 30}"#).unwrap();
        if let JsonValue::Object(obj) = value {
            assert_eq!(
                obj.get("name"),
                Some(&JsonValue::String("Alice".to_string()))
            );
            assert_eq!(obj.get("age"), Some(&JsonValue::Number(30.0)));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_parse_nested_object() {
        let value = parse_json(r#"{"outer": {"inner": 1}}"#).unwrap();
        if let JsonValue::Object(outer) = value {
            if let Some(JsonValue::Object(inner)) = outer.get("outer") {
                assert_eq!(inner.get("inner"), Some(&JsonValue::Number(1.0)));
            } else {
                panic!("Expected nested object");
            }
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_parse_array_in_object() {
        let value = parse_json(r#"{"items": [1, 2, 3]}"#).unwrap();
        if let JsonValue::Object(obj) = value {
            if let Some(JsonValue::Array(arr)) = obj.get("items") {
                assert_eq!(arr.len(), 3);
            } else {
                panic!("Expected array");
            }
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_parse_object_in_array() {
        let value = parse_json(r#"[{"a": 1}, {"b": 2}]"#).unwrap();
        if let JsonValue::Array(arr) = value {
            assert_eq!(arr.len(), 2);
        } else {
            panic!("Expected array");
        }
    }

    // #[test]
    // fn test_object_accessor() {
    //     let value = parse_json(r#"{"name": "test"}"#).unwrap();
    //     let obj = value.as_object().unwrap();
    //     assert_eq!(obj.len(), 1);
    // }

    // #[test]
    // fn test_object_get() {
    //     let value = parse_json(r#"{"name": "Alice", "age": 30}"#).unwrap();
    //     assert_eq!(value.get("name"), Some(&JsonValue::String("Alice".to_string())));
    //     assert_eq!(value.get("missing"), None);
    // }
}
