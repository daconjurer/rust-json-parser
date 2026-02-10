use crate::error::JsonError;

/*
 * Enum for Token kind. Valid variants:
 * LeftBrace, RightBrace, LeftBracket, RightBracket, Comma, Colon
 * String(String), Number(f64), Boolean(bool), Null
 */
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

fn unexpected_token_error<T>(found: String, position: usize) -> Result<T, JsonError> {
    Err(JsonError::UnexpectedToken {
        expected: "valid JSON token".to_string(),
        found,
        position,
    })
}

fn resolve_escape_sequence(char: &char) -> Option<char> {
    match char {
        'n' => Some('\n'),
        't' => Some('\t'),
        'r' => Some('\r'),
        '\\' => Some('\\'),
        '"' => Some('"'),
        '/' => Some('/'),
        'b' => Some('\u{0008}'), // backspace
        'f' => Some('\u{000C}'), // form feed
        _ => None,
    }
}

pub struct Tokenizer {
    input: Vec<char>,
    current: usize,
}

impl Tokenizer {
    pub fn new(input: &str) -> Self {
        Self {
            current: 0,
            input: input.chars().collect(),
        }
    }

    /*
     * Look at current char without advancing
     */
    fn peek(&self) -> Option<char> {
        self.input.get(self.current).copied()
    }

    /*
     * Move forward, return previous char
     */
    fn advance(&mut self) -> Option<char> {
        self.current += 1;
        self.input.get(self.current - 1).copied()
    }

    /*
     * Check if the input has been consumed
     */
    fn is_at_end(&self) -> bool {
        self.peek().is_none()
    }

    fn consume_number(&mut self) -> Result<f64, JsonError> {
        let mut buffer: Vec<char> = Vec::new();

        while let Some(c) = self.peek() {
            if !(c.is_numeric() || c == '.' || c == '-' || c == 'e' || c == 'E' || c == '+') {
                break;
            }
            buffer.push(c);
            self.advance();
        }
        let number_as_string = buffer.iter().collect::<String>();
        let number = number_as_string
            .parse::<f64>()
            .map_err(|_| JsonError::InvalidNumber {
                value: number_as_string.clone(),
                position: 0,
            })?;
        Ok(number)
    }

    fn consume_string(&mut self) -> Result<String, JsonError> {
        let mut buffer: Vec<char> = Vec::new();

        while let Some(c) = self.peek() {
            match c {
                '"' => {
                    self.advance(); // consume closing quote
                    break;
                }
                '\\' => {
                    self.advance(); // consume escape character
                    let special_meaning =
                        self.advance().ok_or(JsonError::UnexpectedEndOfInput {
                            expected: "Special meaning char for escape sequence".to_string(),
                            position: self.current,
                        })?;
                    let escape_sequence = resolve_escape_sequence(&special_meaning).ok_or(
                        JsonError::InvalidEscape {
                            char: special_meaning,
                            position: self.current,
                        },
                    )?;
                    buffer.push(escape_sequence);
                }
                _ => {
                    buffer.push(c);
                    self.advance();
                }
            }
        }
        Ok(buffer.iter().collect::<String>())
    }

    fn consume_keyword(&mut self) -> Result<Token, JsonError> {
        let mut buffer: Vec<char> = Vec::new();

        while let Some(c) = self.peek() {
            if c == ',' || c == ' ' || c == '}' || c == '\n' || c == '\t' {
                break;
            }
            buffer.push(c);
            self.advance();
        }
        let consumed_keyword = buffer.iter().collect::<String>();

        match consumed_keyword.as_str() {
            "true" => Ok(Token::Boolean(true)),
            "false" => Ok(Token::Boolean(false)),
            "null" => Ok(Token::Null),
            _ => {
                let found = match consumed_keyword.chars().next() {
                    Some(first) => first.to_string(),
                    None => "unknown".to_string(),
                };
                unexpected_token_error(found, 0)
            }
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, JsonError> {
        let mut tokens: Vec<Token> = Vec::new();

        while let Some(c) = self.peek() {
            match c {
                ' ' | '\n' | '\t' | '\r' => {
                    self.advance(); // explicitly skip whitespace
                }
                '"' => {
                    self.advance(); // consume opening quote
                    let consumed_string = self.consume_string()?;
                    tokens.push(Token::String(consumed_string));
                }
                '0'..='9' | '-' => {
                    let consumed_number = self.consume_number()?;
                    tokens.push(Token::Number(consumed_number));
                }
                '{' => {
                    self.advance();
                    tokens.push(Token::LeftBrace);
                }
                '}' => {
                    self.advance();
                    tokens.push(Token::RightBrace);
                }
                ',' => {
                    self.advance();
                    tokens.push(Token::Comma);
                }
                ':' => {
                    self.advance();
                    tokens.push(Token::Colon);
                }
                _ if c.is_alphabetic() => {
                    let keyword_token = self.consume_keyword()?;
                    tokens.push(keyword_token);
                }
                _ => {
                    if c.is_ascii_punctuation() {
                        return unexpected_token_error(c.to_string(), 0);
                    }
                    self.advance();
                }
            }
        }

        Ok(tokens)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::error::JsonError;

//     // Result type alias for cleaner test signatures
//     type Result<T> = std::result::Result<T, JsonError>;

//     #[test]
//     fn test_empty_braces() {
//         let tokens = tokenize("{}").expect("Tokenize should process empty brackets");
//         assert_eq!(tokens.len(), 2);
//         assert_eq!(tokens[0], Token::LeftBrace);
//         assert_eq!(tokens[1], Token::RightBrace);
//     }

//     #[test]
//     fn test_simple_string() {
//         let tokens =
//             tokenize(r#""hello""#).expect("Tokenize should process simple raw string literals");
//         assert_eq!(tokens.len(), 1);
//         assert_eq!(tokens[0], Token::String("hello".to_string()));
//     }

//     #[test]
//     fn test_number() {
//         let tokens = tokenize("42").expect("Tokenize should process simple number");
//         assert_eq!(tokens.len(), 1);
//         assert_eq!(tokens[0], Token::Number(42.0));
//     }

//     #[test]
//     fn test_tokenize_string() {
//         let tokens =
//             tokenize(r#""hello world""#).expect("Tokenize should process strings with spaces");

//         assert_eq!(tokens.len(), 1);
//         assert_eq!(tokens[0], Token::String("hello world".to_string()));
//     }

//     #[test]
//     fn test_boolean_and_null() {
//         let tokens: Vec<Token> = tokenize("true false null")
//             .expect("Tokenize should process keywords (null, false, true)");
//         assert_eq!(tokens.len(), 3);
//         assert_eq!(tokens[0], Token::Boolean(true));
//         assert_eq!(tokens[1], Token::Boolean(false));
//         assert_eq!(tokens[2], Token::Null);
//     }

//     #[test]
//     fn test_simple_object() {
//         let tokens: Vec<Token> =
//             tokenize(r#"{"name": "Alice"}"#).expect("Tokenize should process simple object");
//         assert_eq!(tokens.len(), 5);
//         assert_eq!(tokens[0], Token::LeftBrace);
//         assert_eq!(tokens[1], Token::String("name".to_string()));
//         assert_eq!(tokens[2], Token::Colon);
//         assert_eq!(tokens[3], Token::String("Alice".to_string()));
//         assert_eq!(tokens[4], Token::RightBrace);
//     }

//     #[test]
//     fn test_multiple_values() {
//         let tokens = tokenize(r#"{"age": 30, "active": true}"#)
//             .expect("Tokenize should process object with multiple values");

//         assert_eq!(tokens.len(), 9);
//         // Verify we have the right tokens
//         assert_eq!(tokens[0], Token::LeftBrace);
//         assert!(tokens.contains(&Token::String("age".to_string())));
//         assert!(tokens.contains(&Token::Number(30.0)));
//         assert!(tokens.contains(&Token::Comma));
//         assert!(tokens.contains(&Token::String("active".to_string())));
//         assert!(tokens.contains(&Token::Boolean(true)));
//         assert_eq!(tokens[8], Token::RightBrace);
//     }

//     /*
//      * Error handling tests
//      */
//     // String boundary tests - verify inner vs outer quote handling
//     #[test]
//     fn test_empty_string() -> Result<()> {
//         // Outer boundary: adjacent quotes with no inner content
//         let tokens = tokenize(r#""""#)?;
//         assert_eq!(tokens.len(), 1);
//         assert_eq!(tokens[0], Token::String("".to_string()));
//         Ok(())
//     }

//     #[test]
//     fn test_string_containing_json_special_chars() -> Result<()> {
//         // Inner handling: JSON delimiters inside strings don't break tokenization
//         let tokens = tokenize(r#""{key: value}""#)?;
//         assert_eq!(tokens.len(), 1);
//         assert_eq!(tokens[0], Token::String("{key: value}".to_string()));
//         Ok(())
//     }

//     #[test]
//     fn test_string_with_keyword_like_content() -> Result<()> {
//         // Inner handling: "true", "false", "null" inside strings stay as string content
//         let tokens = tokenize(r#""not true or false""#)?;
//         assert_eq!(tokens.len(), 1);
//         assert_eq!(tokens[0], Token::String("not true or false".to_string()));
//         Ok(())
//     }

//     #[test]
//     fn test_string_with_number_like_content() -> Result<()> {
//         // Inner handling: numeric content inside strings doesn't become Number tokens
//         let tokens = tokenize(r#""phone: 555-1234""#)?;
//         assert_eq!(tokens.len(), 1);
//         assert_eq!(tokens[0], Token::String("phone: 555-1234".to_string()));
//         Ok(())
//     }

//     // Number parsing tests
//     #[test]
//     fn test_negative_number() -> Result<()> {
//         let tokens = tokenize("-42")?;
//         assert_eq!(tokens.len(), 1);
//         assert_eq!(tokens[0], Token::Number(-42.0));
//         Ok(())
//     }

//     #[test]
//     fn test_decimal_number() -> Result<()> {
//         let tokens = tokenize("0.5")?;
//         assert_eq!(tokens.len(), 1);
//         assert_eq!(tokens[0], Token::Number(0.5));
//         Ok(())
//     }

//     #[test]
//     fn test_leading_decimal_not_a_number() {
//         // .5 is invalid JSON - numbers must have leading digit (0.5 is valid)
//         let result = tokenize(".5");
//         // Should NOT be interpreted as 0.5
//         assert!(matches!(result, Err(JsonError::UnexpectedToken { .. })));
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    // === Struct Usage Tests ===

    #[test]
    fn test_tokenizer_struct_creation() {
        let _ = Tokenizer::new(r#""hello""#);
        // Tokenizer should be created without error
        // Internal state is private, so we test via tokenize()
    }

    #[test]
    fn test_tokenizer_multiple_tokens() {
        // Tests that a single tokenize() call handles multiple tokens
        // Note: Unlike Python iterators, calling tokenize() again on the same
        // instance would return empty - the input has been consumed.
        // Create a new Tokenizer instance if you need to parse new input.
        let mut tokenizer = Tokenizer::new("123 456");
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2);
    }

    // === Basic Token Tests (from Week 1 - ensure they still pass) ===

    #[test]
    fn test_tokenize_number() {
        let mut tokenizer = Tokenizer::new("42");
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::Number(42.0)]);
    }

    #[test]
    fn test_tokenize_negative_number() {
        let mut tokenizer = Tokenizer::new("-3.14");
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::Number(-3.14)]);
    }

    #[test]
    fn test_tokenize_literals() {
        let mut t1 = Tokenizer::new("true");
        assert_eq!(t1.tokenize().unwrap(), vec![Token::Boolean(true)]);

        let mut t2 = Tokenizer::new("false");
        assert_eq!(t2.tokenize().unwrap(), vec![Token::Boolean(false)]);

        let mut t3 = Tokenizer::new("null");
        assert_eq!(t3.tokenize().unwrap(), vec![Token::Null]);
    }

    #[test]
    fn test_tokenize_simple_string() {
        let mut tokenizer = Tokenizer::new(r#""hello""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("hello".to_string())]);
    }

    // === Escape Sequence Tests ===

    #[test]
    fn test_escape_newline() {
        let mut tokenizer = Tokenizer::new(r#""hello\nworld""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("hello\nworld".to_string())]);
    }

    #[test]
    fn test_escape_tab() {
        let mut tokenizer = Tokenizer::new(r#""col1\tcol2""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("col1\tcol2".to_string())]);
    }

    #[test]
    fn test_escape_quote() {
        let mut tokenizer = Tokenizer::new(r#""say \"hello\"""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("say \"hello\"".to_string())]);
    }

    #[test]
    fn test_escape_backslash() {
        let mut tokenizer = Tokenizer::new(r#""path\\to\\file""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("path\\to\\file".to_string())]);
    }

    #[test]
    fn test_escape_forward_slash() {
        let mut tokenizer = Tokenizer::new(r#""a\/b""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("a/b".to_string())]);
    }

    #[test]
    fn test_escape_carriage_return() {
        let mut tokenizer = Tokenizer::new(r#""line\r\n""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("line\r\n".to_string())]);
    }

    #[test]
    fn test_escape_backspace_formfeed() {
        let mut tokenizer = Tokenizer::new(r#""\b\f""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("\u{0008}\u{000C}".to_string())]);
    }

    #[test]
    fn test_multiple_escapes() {
        let mut tokenizer = Tokenizer::new(r#""a\nb\tc\"""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("a\nb\tc\"".to_string())]);
    }

    // === Unicode Escape Tests ===

    // #[test]
    // fn test_unicode_escape_basic() {
    //     // \u0041 is 'A'
    //     let mut tokenizer = Tokenizer::new(r#""\u0041""#);
    //     let tokens = tokenizer.tokenize().unwrap();
    //     assert_eq!(tokens, vec![Token::String("A".to_string())]);
    // }

    // #[test]
    // fn test_unicode_escape_multiple() {
    //     // \u0048\u0069 is "Hi"
    //     let mut tokenizer = Tokenizer::new(r#""\u0048\u0069""#);
    //     let tokens = tokenizer.tokenize().unwrap();
    //     assert_eq!(tokens, vec![Token::String("Hi".to_string())]);
    // }

    // #[test]
    // fn test_unicode_escape_mixed() {
    //     // Mix of regular chars and unicode escapes
    //     let mut tokenizer = Tokenizer::new(r#""Hello \u0057orld""#);
    //     let tokens = tokenizer.tokenize().unwrap();
    //     assert_eq!(tokens, vec![Token::String("Hello World".to_string())]);
    // }

    // #[test]
    // fn test_unicode_escape_lowercase() {
    //     // Lowercase hex digits should work too
    //     let mut tokenizer = Tokenizer::new(r#""\u004a""#);
    //     let tokens = tokenizer.tokenize().unwrap();
    //     assert_eq!(tokens, vec![Token::String("J".to_string())]);
    // }

    // === Error Tests ===

    #[test]
    fn test_invalid_escape_sequence() {
        let mut tokenizer = Tokenizer::new(r#""\q""#);
        let result = tokenizer.tokenize();
        assert!(matches!(result, Err(JsonError::InvalidEscape { .. })));
    }

    // #[test]
    // fn test_invalid_unicode_too_short() {
    //     let mut tokenizer = Tokenizer::new(r#""\u004""#);
    //     let result = tokenizer.tokenize();
    //     assert!(matches!(result, Err(JsonError::InvalidUnicode { .. })));
    // }

    // #[test]
    // fn test_invalid_unicode_bad_hex() {
    //     let mut tokenizer = Tokenizer::new(r#""\u00GG""#);
    //     let result = tokenizer.tokenize();
    //     assert!(matches!(result, Err(JsonError::InvalidUnicode { .. })));
    // }

    // #[test]
    // fn test_unterminated_string_with_escape() {
    //     let mut tokenizer = Tokenizer::new(r#""hello\n"#);
    //     let result = tokenizer.tokenize();
    //     assert!(result.is_err());
    // }
}
