use crate::error::unexpected_token_error;
use crate::{JsonError, JsonResult};

fn resolve_escape_sequence(char: char) -> Option<char> {
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

/// Represents a Token result of tokenization
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// A quoted string value.
    String(String),
    /// A numeric literal.
    Number(f64),
    /// A `true` or `false` literal.
    Boolean(bool),
    /// The `null` literal.
    Null,

    /// Opening bracket `[`.
    LeftBracket,
    /// Closing bracket `]`.
    RightBracket,
    /// Opening brace `{`.
    LeftBrace,
    /// Closing brace `}`.
    RightBrace,
    /// Colon `:` separating keys from values.
    Colon,
    /// Comma `,` separating elements.
    Comma,
}

impl Token {
    /// Returns `true` if `self` and `other` are the same variant, ignoring inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_json_parser::Token;
    ///
    /// let a = Token::String("hello".to_string());
    /// let b = Token::String("world".to_string());
    /// assert!(a.is_variant(&b));
    ///
    /// let c = Token::Number(42.0);
    /// assert!(!a.is_variant(&c));
    /// ```
    pub fn is_variant(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

fn parse_unicode_hex(s: &str) -> Option<char> {
    if s.len() != 4 {
        return None;
    }
    u32::from_str_radix(s, 16).ok().and_then(char::from_u32)
}

/// A lexer that converts a JSON input string into a sequence of [`Token`]s.
pub struct Tokenizer<'input> {
    input: &'input str,
    current: usize,
}

impl<'input> Tokenizer<'input> {
    /// Creates a new `Tokenizer` for the given JSON input string.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_json_parser::Tokenizer;
    ///
    /// let tokenizer = Tokenizer::new(r#"{"key": 42}"#);
    /// ```
    pub fn new(input: &'input str) -> Self {
        Self {
            current: 0,
            input: input,
        }
    }

    /*
     * Look at current byte
     */
    fn peek(&self) -> Option<&u8> {
        self.input.as_bytes().get(self.current)
    }

    /*
     * Move forward, return previous byte
     */
    fn advance(&mut self) -> Option<&u8> {
        let b = self.input.as_bytes().get(self.current)?;
        self.current += 1;
        Some(b)
    }

    fn _input_slice_to_string(&self, start: usize, end: usize) -> String {
        self.input[start..end].to_string()
    }

    /*
     * Check if the input has been consumed
     */
    fn _is_at_end(&self) -> bool {
        self.peek().is_none()
    }

    fn consume_number(&mut self) -> JsonResult<f64> {
        let start = self.current;

        while let Some(c) = self.peek() {
            if !(c.is_ascii_digit()
                || *c == b'.'
                || *c == b'-'
                || *c == b'e'
                || *c == b'E'
                || *c == b'+')
            {
                break;
            }
            self.advance();
        }
        let slice = &self.input[start..self.current];
        let number = slice.parse::<f64>().map_err(|_| JsonError::InvalidNumber {
            value: slice.to_string(),
            position: self.current,
        })?;
        Ok(number)
    }

    fn consume_string(&mut self) -> JsonResult<String> {
        let start = self.current;

        // Fast path: scan for closing quote with no escape sequences
        loop {
            match self.peek() {
                Some(b'"') => {
                    let s = self._input_slice_to_string(start, self.current);
                    self.advance(); // Consume closing quote
                    return Ok(s);
                }
                Some(b'\\') => {
                    // Copy what we've scanned so far and switch to slow path
                    let mut s: String = self._input_slice_to_string(start, self.current);
                    return self.consume_string_slow(&mut s);
                }
                Some(_) => {
                    self.advance();
                }
                None => {
                    return Err(JsonError::UnexpectedEndOfInput {
                        expected: "Closing quote".to_string(),
                        position: self.current,
                    });
                }
            }
        }
    }

    fn consume_string_slow(&mut self, s: &mut String) -> JsonResult<String> {
        while let Some(&b) = self.peek() {
            match b {
                b'"' => {
                    self.advance();
                    return Ok(std::mem::take(s));
                }
                b'\\' => {
                    self.advance(); // consume escape character
                    let special_meaning =
                        self.advance()
                            .copied()
                            .ok_or(JsonError::UnexpectedEndOfInput {
                                expected: "Special meaning char for escape sequence".to_string(),
                                position: self.current,
                            })?;

                    if special_meaning == b'u' {
                        let hex_start = self.current;
                        if self.current + 4 > self.input.len() {
                            return Err(JsonError::InvalidUnicode {
                                sequence: format!("\\u{}", &self.input[hex_start..]),
                                position: self.current,
                            });
                        }
                        let hex_str = &self.input[hex_start..hex_start + 4];
                        let ch = parse_unicode_hex(hex_str).ok_or(JsonError::InvalidUnicode {
                            sequence: format!("\\u{}", hex_str),
                            position: self.current,
                        })?;
                        s.push(ch);
                        self.current += 4;
                    } else {
                        let ch = resolve_escape_sequence(special_meaning as char).ok_or(
                            JsonError::InvalidEscape {
                                char: special_meaning as char,
                                position: self.current,
                            },
                        )?;
                        s.push(ch);
                    }
                }
                _ => {
                    s.push(b as char);
                    self.advance();
                }
            }
        }
        // Unterminated string
        Err(JsonError::UnexpectedEndOfInput {
            expected: "Closing quote".to_string(),
            position: self.current,
        })
    }

    fn consume_keyword(&mut self) -> JsonResult<Token> {
        let start = self.current;

        while let Some(c) = self.peek() {
            if !c.is_ascii_alphabetic() {
                break;
            }
            self.advance();
        }

        let slice = &self.input[start..self.current];
        match slice {
            "true" => Ok(Token::Boolean(true)),
            "false" => Ok(Token::Boolean(false)),
            "null" => Ok(Token::Null),
            _ => {
                let found = match slice.chars().next() {
                    Some(first) => first.to_string(),
                    None => "unknown".to_string(),
                };
                Err(unexpected_token_error("Valid JSON value", &found, 0))
            }
        }
    }

    /// Consumes the input and returns the complete list of tokens.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_json_parser::{Tokenizer, Token};
    ///
    /// let mut tokenizer = Tokenizer::new("[1, true]");
    /// let tokens = tokenizer.tokenize()?;
    /// assert_eq!(tokens, vec![
    ///     Token::LeftBracket,
    ///     Token::Number(1.0),
    ///     Token::Comma,
    ///     Token::Boolean(true),
    ///     Token::RightBracket,
    /// ]);
    /// # Ok::<(), rust_json_parser::JsonError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`JsonError::UnexpectedToken`] if an invalid character is encountered,
    /// [`JsonError::InvalidNumber`] if a numeric literal cannot be parsed,
    /// [`JsonError::InvalidEscape`] if a string contains an unrecognized escape sequence,
    /// [`JsonError::InvalidUnicode`] if a `\uXXXX` sequence is malformed, or
    /// [`JsonError::UnexpectedEndOfInput`] if a string is unterminated.
    pub fn tokenize(&mut self) -> JsonResult<Vec<Token>> {
        let mut tokens: Vec<Token> = Vec::new();

        while let Some(c) = self.peek() {
            match c {
                b' ' | b'\n' | b'\t' | b'\r' => {
                    self.advance(); // explicitly skip whitespace
                }
                b'"' => {
                    self.advance(); // consume opening quote
                    let consumed_string = self.consume_string()?;
                    tokens.push(Token::String(consumed_string));
                }
                b'0'..=b'9' | b'-' => {
                    let consumed_number = self.consume_number()?;
                    tokens.push(Token::Number(consumed_number));
                }
                b'{' => {
                    self.advance();
                    tokens.push(Token::LeftBrace);
                }
                b'}' => {
                    self.advance();
                    tokens.push(Token::RightBrace);
                }
                b'[' => {
                    self.advance();
                    tokens.push(Token::LeftBracket);
                }
                b']' => {
                    self.advance();
                    tokens.push(Token::RightBracket);
                }
                b',' => {
                    self.advance();
                    tokens.push(Token::Comma);
                }
                b':' => {
                    self.advance();
                    tokens.push(Token::Colon);
                }
                _ if c.is_ascii_alphabetic() => {
                    let keyword_token = self.consume_keyword()?;
                    tokens.push(keyword_token);
                }
                _ => {
                    if c.is_ascii_punctuation() {
                        return Err(unexpected_token_error(
                            "Valid JSON value",
                            &(*c as char).to_string(),
                            0,
                        ));
                    }
                    self.advance();
                }
            }
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::JsonError;

    // === Struct Usage Tests ===

    #[test]
    fn test_tokenizer_struct_creation() {
        let _ = Tokenizer::new(r#""hello""#);
        // Tokenizer should be created without error
        // Internal state is private, so we test via tokenize()
    }

    #[test]
    fn test_empty_braces() {
        let mut tokenizer = Tokenizer::new("{}");
        let tokens = tokenizer
            .tokenize()
            .expect("Tokenize should process empty brackets");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::LeftBrace);
        assert_eq!(tokens[1], Token::RightBrace);
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
    fn test_decimal_number() {
        let mut tokenizer = Tokenizer::new("0.5");
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(0.5));
    }

    #[test]
    fn test_leading_decimal_not_a_number() {
        // .5 is invalid JSON - numbers must have leading digit (0.5 is valid)
        let mut tokenizer = Tokenizer::new(".5");
        let result = tokenizer.tokenize();
        // Should NOT be interpreted as 0.5
        assert!(matches!(result, Err(JsonError::UnexpectedToken { .. })));
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

    #[test]
    fn test_simple_object() {
        let mut tokenizer = Tokenizer::new(r#"{"name": "Alice"}"#);
        let tokens = tokenizer
            .tokenize()
            .expect("Tokenize should process simple object");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], Token::LeftBrace);
        assert_eq!(tokens[1], Token::String("name".to_string()));
        assert_eq!(tokens[2], Token::Colon);
        assert_eq!(tokens[3], Token::String("Alice".to_string()));
        assert_eq!(tokens[4], Token::RightBrace);
    }

    #[test]
    fn test_multiple_values() {
        let mut tokenizer = Tokenizer::new(r#"{"age": 30, "active": true}"#);
        let tokens = tokenizer
            .tokenize()
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

    // String boundary tests - verify inner vs outer quote handling
    #[test]
    fn test_empty_string() {
        // Outer boundary: adjacent quotes with no inner content
        let mut tokenizer = Tokenizer::new(r#""""#);
        let tokens = tokenizer
            .tokenize()
            .expect("Tokenize should process adjacent quotes with no inner content");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("".to_string()));
    }

    #[test]
    fn test_string_containing_json_special_chars() {
        // Inner handling: JSON delimiters inside strings don't break tokenization
        let mut tokenizer = Tokenizer::new(r#""{key: value}""#);
        let tokens = tokenizer
            .tokenize()
            .expect("Tokenizer should process JSON delimiters inside string");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("{key: value}".to_string()));
    }

    #[test]
    fn test_string_with_keyword_like_content() {
        // Inner handling: "true", "false", "null" inside strings stay as string content
        let mut tokenizer = Tokenizer::new(r#""not true or false""#);
        let tokens = tokenizer
            .tokenize()
            .expect("Tokenizer should handle keywords as string content");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("not true or false".to_string()));
    }

    #[test]
    fn test_string_with_number_like_content() {
        // Inner handling: numeric content inside strings doesn't become Number tokens
        let mut tokenizer = Tokenizer::new(r#""phone: 555-1234""#);
        let tokens = tokenizer
            .tokenize()
            .expect("Tokenizer should handle numeric content inside string");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("phone: 555-1234".to_string()));
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
