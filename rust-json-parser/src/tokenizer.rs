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
        if c == ',' || c == ' ' {
            break;
        }
        buffer.push(c);
        chars.next();
    }
    let number_as_string = buffer.iter().collect::<String>();
    let number = number_as_string.parse::<f64>()?;
    Ok(number)
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, JsonError> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
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
                // TODO: Narrow me
                if c.is_ascii_punctuation() {
                    return unexpected_token_error(c.to_string(), 0);
                }
                chars.next();
            } // TODO: raise error
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::JsonError;

    // Result type alias for cleaner test signatures
    type Result<T> = std::result::Result<T, JsonError>;

    #[test]
    fn test_empty_braces() {
        let tokens = tokenize("{}").expect("Tokenize should process empty brackets");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::LeftBrace);
        assert_eq!(tokens[1], Token::RightBrace);
    }

    #[test]
    fn test_simple_string() {
        let tokens =
            tokenize(r#""hello""#).expect("Tokenize should process simple raw string literals");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("hello".to_string()));
    }

    #[test]
    fn test_number() {
        let tokens = tokenize("42").expect("Tokenize should process simple number");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(42.0));
    }

    #[test]
    fn test_tokenize_string() {
        let tokens =
            tokenize(r#""hello world""#).expect("Tokenize should process strings with spaces");

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("hello world".to_string()));
    }

    #[test]
    fn test_boolean_and_null() {
        let tokens: Vec<Token> = tokenize("true false null")
            .expect("Tokenize should process keywords (null, false, true)");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Boolean(true));
        assert_eq!(tokens[1], Token::Boolean(false));
        assert_eq!(tokens[2], Token::Null);
    }

    #[test]
    fn test_simple_object() {
        let tokens: Vec<Token> =
            tokenize(r#"{"name": "Alice"}"#).expect("Tokenize should process simple object");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], Token::LeftBrace);
        assert_eq!(tokens[1], Token::String("name".to_string()));
        assert_eq!(tokens[2], Token::Colon);
        assert_eq!(tokens[3], Token::String("Alice".to_string()));
        assert_eq!(tokens[4], Token::RightBrace);
    }

    #[test]
    fn test_multiple_values() {
        let tokens = tokenize(r#"{"age": 30, "active": true}"#)
            .expect("Tokenize should process object with multiple values");

        assert_eq!(tokens.len(), 9);
        // Verify we have the right tokens
        assert_eq!(tokens[0], Token::LeftBrace);
        assert!(tokens.contains(&Token::String("age".to_string())));
        assert!(tokens.contains(&Token::Number(30.0)));
        assert!(tokens.contains(&Token::Comma));
        assert!(tokens.contains(&Token::String("active".to_string())));
        assert!(tokens.contains(&Token::Boolean(true)));
        assert_eq!(tokens[8], Token::RightBrace);
    }

    /*
     * Error handling tests
     */

    // String boundary tests - verify inner vs outer quote handling
    #[test]
    fn test_empty_string() -> Result<()> {
        // Outer boundary: adjacent quotes with no inner content
        let tokens = tokenize(r#""""#)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("".to_string()));
        Ok(())
    }

    #[test]
    fn test_string_containing_json_special_chars() -> Result<()> {
        // Inner handling: JSON delimiters inside strings don't break tokenization
        let tokens = tokenize(r#""{key: value}""#)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("{key: value}".to_string()));
        Ok(())
    }

    #[test]
    fn test_string_with_keyword_like_content() -> Result<()> {
        // Inner handling: "true", "false", "null" inside strings stay as string content
        let tokens = tokenize(r#""not true or false""#)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("not true or false".to_string()));
        Ok(())
    }

    #[test]
    fn test_string_with_number_like_content() -> Result<()> {
        // Inner handling: numeric content inside strings doesn't become Number tokens
        let tokens = tokenize(r#""phone: 555-1234""#)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("phone: 555-1234".to_string()));
        Ok(())
    }

    // Number parsing tests
    #[test]
    fn test_negative_number() -> Result<()> {
        let tokens = tokenize("-42")?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(-42.0));
        Ok(())
    }

    #[test]
    fn test_decimal_number() -> Result<()> {
        let tokens = tokenize("0.5")?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(0.5));
        Ok(())
    }

    #[test]
    fn test_leading_decimal_not_a_number() {
        // .5 is invalid JSON - numbers must have leading digit (0.5 is valid)
        let result = tokenize(".5");
        // Should NOT be interpreted as 0.5
        assert!(matches!(result, Err(JsonError::UnexpectedToken { .. })));
    }
}
