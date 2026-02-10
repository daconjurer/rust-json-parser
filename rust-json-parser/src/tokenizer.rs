use crate::error::JsonError;
use std::iter::Peekable;
use std::str::Chars;

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

fn consume_keyword(chars: &mut Peekable<Chars>) -> Result<Token, JsonError> {
    let mut buffer: Vec<char> = Vec::new();

    while let Some(&c) = chars.peek() {
        if c == ',' || c == ' ' || c == '}' || c == '\n' || c == '\t' {
            break;
        }
        buffer.push(c);
        chars.next();
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

fn consume_string(chars: &mut Peekable<Chars>) -> String {
    let mut buffer: Vec<char> = Vec::new();

    while let Some(&c) = chars.peek() {
        if c == '"' {
            chars.next(); // consume closing quote
            break;
        }
        buffer.push(c);
        chars.next();
    }
    buffer.iter().collect::<String>()
}

fn consume_number(chars: &mut Peekable<Chars>) -> Result<f64, JsonError> {
    let mut buffer: Vec<char> = Vec::new();

    while let Some(&c) = chars.peek() {
        if !(c.is_numeric() || c == '.' || c == '-' || c == 'e' || c == 'E' || c == '+') {
            break;
        }
        buffer.push(c);
        chars.next();
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

pub fn tokenize(input: &str) -> Result<Vec<Token>, JsonError> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\n' | '\t' | '\r' => {
                chars.next(); // explicitly skip whitespace
            }
            '"' => {
                chars.next(); // consume opening quote
                let consumed_string = consume_string(&mut chars);
                tokens.push(Token::String(consumed_string));
            }
            '0'..='9' | '-' => {
                let n = consume_number(&mut chars)?;
                tokens.push(Token::Number(n));
            }
            '{' => {
                chars.next();
                tokens.push(Token::LeftBrace);
            }
            '}' => {
                chars.next();
                tokens.push(Token::RightBrace);
            }
            ',' => {
                chars.next();
                tokens.push(Token::Comma);
            }
            ':' => {
                chars.next();
                tokens.push(Token::Colon);
            }
            _ if c.is_alphabetic() => {
                let keyword_token = consume_keyword(&mut chars)?;
                tokens.push(keyword_token);
            }
            _ => {
                if c.is_ascii_punctuation() {
                    return unexpected_token_error(c.to_string(), 0);
                }
                chars.next();
            }
        }
    }

    Ok(tokens)
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
        let tokenizer = Tokenizer::new(r#""hello""#);
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

    #[test]
    fn test_unicode_escape_basic() {
        // \u0041 is 'A'
        let mut tokenizer = Tokenizer::new(r#""\u0041""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("A".to_string())]);
    }

    #[test]
    fn test_unicode_escape_multiple() {
        // \u0048\u0069 is "Hi"
        let mut tokenizer = Tokenizer::new(r#""\u0048\u0069""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("Hi".to_string())]);
    }

    #[test]
    fn test_unicode_escape_mixed() {
        // Mix of regular chars and unicode escapes
        let mut tokenizer = Tokenizer::new(r#""Hello \u0057orld""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("Hello World".to_string())]);
    }

    #[test]
    fn test_unicode_escape_lowercase() {
        // Lowercase hex digits should work too
        let mut tokenizer = Tokenizer::new(r#""\u004a""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("J".to_string())]);
    }

    // === Error Tests ===

    #[test]
    fn test_invalid_escape_sequence() {
        let mut tokenizer = Tokenizer::new(r#""\q""#);
        let result = tokenizer.tokenize();
        assert!(matches!(result, Err(JsonError::InvalidEscape { .. })));
    }

    #[test]
    fn test_invalid_unicode_too_short() {
        let mut tokenizer = Tokenizer::new(r#""\u004""#);
        let result = tokenizer.tokenize();
        assert!(matches!(result, Err(JsonError::InvalidUnicode { .. })));
    }

    #[test]
    fn test_invalid_unicode_bad_hex() {
        let mut tokenizer = Tokenizer::new(r#""\u00GG""#);
        let result = tokenizer.tokenize();
        assert!(matches!(result, Err(JsonError::InvalidUnicode { .. })));
    }

    #[test]
    fn test_unterminated_string_with_escape() {
        let mut tokenizer = Tokenizer::new(r#""hello\n"#);
        let result = tokenizer.tokenize();
        assert!(result.is_err());
    }
}
