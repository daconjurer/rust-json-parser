use std::collections::HashMap;

use crate::error::{unexpected_end_of_input, unexpected_token_error};
use crate::tokenizer::{Token, Tokenizer};
use crate::value::JsonValue;
use crate::{JsonError, JsonResult};
use std::fs;

/*
 * Utility function to error upon missing expected comma
*/
fn err_on_missing_expected_comma(
    expected_comma: bool,
    found: &Token,
    position: usize,
) -> JsonResult<()> {
    if expected_comma {
        return Err(unexpected_token_error(
            ",",
            &format!("{:?}", found),
            position,
        ));
    }
    Ok(())
}

/*
 * Utility function to error upon finding an unexpected JSON value before a colon
*/
fn err_on_unexpected_value_before_colon(
    colon_found: bool,
    found: &str,
    position: usize,
) -> JsonResult<()> {
    if !colon_found {
        return Err(unexpected_token_error("string", found, position));
    }
    Ok(())
}

/*
 * Utility function to check if the next token is an expected colon
*/
fn next_token_is_expected_colon(
    colon_found: bool,
    next_token: &Token,
    position: usize,
) -> JsonResult<bool> {
    if colon_found {
        return Ok(false);
    }

    if next_token != &Token::Colon {
        return Err(unexpected_token_error(
            ":",
            &format!("{:?}", next_token),
            position,
        ));
    }

    Ok(true)
}

/*
 * Utility function to error upon finding unexpected comma
*/
fn err_on_unexpected_comma(
    expected_comma: bool,
    expected: &str,
    position: usize,
) -> JsonResult<()> {
    if !expected_comma {
        return Err(unexpected_token_error(expected, ",", position));
    }
    Ok(())
}

/*
 * Utility function to error upon finding unexpected closing token
*/
fn err_on_unexpected_closing_token(
    token: &Token,
    expected_token: &Token,
    expected: &str,
    found: &str,
    position: usize,
) -> JsonResult<()> {
    if token.is_variant(expected_token) {
        return Err(unexpected_token_error(expected, found, position));
    }
    Ok(())
}

/// A recursive descent parser that converts a token stream into a [`JsonValue`] tree.
pub struct JsonParser {
    tokens: Vec<(Token, usize)>,
    current: usize,
    input_len: usize,
}

impl JsonParser {
    /// Tokenizes the input string and creates a new `JsonParser` ready to parse.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_json_parser::JsonParser;
    ///
    /// let parser = JsonParser::new(r#"{"key": "value"}"#)?;
    /// # Ok::<(), rust_json_parser::JsonError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`JsonError`] if the input contains invalid tokens,
    /// reported with the byte position of the offending character.
    pub fn new(input: &str) -> JsonResult<Self> {
        let mut tokenizer = Tokenizer::new(input);
        let tokens = tokenizer.tokenize()?;
        Ok(Self {
            current: 0,
            tokens,
            input_len: input.len(),
        })
    }

    /// Parses the token stream and returns the root [`JsonValue`].
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_json_parser::{JsonParser, JsonValue};
    ///
    /// let mut parser = JsonParser::new("[1, 2, 3]")?;
    /// let value = parser.parse()?;
    /// assert_eq!(value.as_array().map(|a| a.len()), Some(3));
    /// # Ok::<(), rust_json_parser::JsonError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`JsonError::UnexpectedToken`] if the
    /// token stream contains structurally invalid JSON (e.g. missing commas, colons, or
    /// mismatched brackets), or
    /// [`JsonError::UnexpectedEndOfInput`] if the
    /// input ends before a complete value is formed.
    pub fn parse(&mut self) -> JsonResult<JsonValue> {
        match self.peek() {
            Some(Token::LeftBrace) => self.parse_object(),
            Some(Token::LeftBracket) => self.parse_array(),
            Some(_) => self.parse_primitive(),
            None => Err(unexpected_end_of_input("string", self.position())),
        }
    }

    /*
     * Parses a JSON primitive type (string, number, boolean or null)
     */
    fn parse_primitive(&mut self) -> JsonResult<JsonValue> {
        match self.peek() {
            Some(Token::String(s)) => Ok(JsonValue::String(s.clone())),
            Some(Token::Number(n)) => Ok(JsonValue::Number(*n)),
            Some(Token::Boolean(b)) => Ok(JsonValue::Boolean(*b)),
            Some(Token::Null) => Ok(JsonValue::Null),
            Some(token) => Err(unexpected_token_error(
                "string",
                &format!("{:?}", token),
                self.position(),
            )),
            None => Err(unexpected_end_of_input("string", self.position())),
        }
    }

    /*
     * Parses an array recursively, handling any valid JSON values.
     *
     * As array nesting produces the following string pattern: "[[", this method
     * requires the opening bracket to be consumed beforehand.
     */
    fn parse_array(&mut self) -> JsonResult<JsonValue> {
        self.advance(); // Consume opening [
        let mut array = Vec::new();
        let mut expect_comma = false;

        while let Some(token) = self.peek() {
            match token {
                // Start of array
                Token::LeftBracket => {
                    err_on_missing_expected_comma(expect_comma, token, self.position())?;

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
                    err_on_missing_expected_comma(expect_comma, token, self.position())?;

                    let nested_object = self.parse_object()?;
                    array.push(nested_object);
                    expect_comma = true;
                }
                Token::String(s) => {
                    err_on_missing_expected_comma(expect_comma, token, self.position())?;

                    array.push(JsonValue::String(s.clone()));
                    self.advance();
                    expect_comma = true;
                }
                Token::Number(n) => {
                    err_on_missing_expected_comma(expect_comma, token, self.position())?;

                    array.push(JsonValue::Number(*n));
                    self.advance();
                    expect_comma = true;
                }
                Token::Boolean(b) => {
                    err_on_missing_expected_comma(expect_comma, token, self.position())?;

                    array.push(JsonValue::Boolean(*b));
                    self.advance();
                    expect_comma = true;
                }
                Token::Null => {
                    err_on_missing_expected_comma(expect_comma, token, self.position())?;

                    array.push(JsonValue::Null);
                    self.advance();
                    expect_comma = true;
                }
                Token::Comma => {
                    self.advance(); // Consume comma
                    let token = self.peek().ok_or(unexpected_end_of_input(
                        "string, bool, number or object",
                        self.position(),
                    ))?;

                    err_on_unexpected_comma(expect_comma, "closing bracket", self.position())?;
                    err_on_unexpected_closing_token(
                        token,
                        &Token::RightBracket,
                        "string, bool, number or object",
                        "]",
                        self.position(),
                    )?;
                    expect_comma = false;
                }
                _ => {
                    return Err(unexpected_token_error(
                        "valid JSON value",
                        &format!("{:?}", token),
                        self.position(),
                    ));
                }
            };
        }

        Err(unexpected_end_of_input("closing bracket", self.position()))
    }

    /*
     * Parses an object recursively, handling any valid JSON values.
     *
     * As object nesting never produces a string pattern: "{{", this method
     * consumes the opening brace.
     */
    fn parse_object(&mut self) -> JsonResult<JsonValue> {
        self.advance(); // Consume opening {
        let mut key = String::new();
        let mut object = HashMap::new();
        let mut colon_found = false;
        let mut expect_comma = false;

        while let Some(token) = self.peek() {
            match token {
                // Start of object
                Token::LeftBrace => {
                    err_on_missing_expected_comma(expect_comma, token, self.position())?;

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
                    err_on_missing_expected_comma(expect_comma, token, self.position())?;

                    if colon_found {
                        let array = self.parse_array()?;
                        object.insert(key.clone(), array);
                        colon_found = false;
                        expect_comma = true;
                    }
                }
                // Key or string value
                Token::String(s) => {
                    err_on_missing_expected_comma(expect_comma, token, self.position())?;

                    // Unexpected end of input
                    let next_token =
                        self.get_token(self.current + 1)
                            .ok_or(unexpected_end_of_input(
                                match colon_found {
                                    true => ",",
                                    false => ":",
                                },
                                self.position(),
                            ))?;

                    // All good! Key?
                    if next_token_is_expected_colon(colon_found, next_token, self.position())? {
                        key = s.clone();
                    // Or value?
                    } else {
                        object.insert(key.clone(), JsonValue::String(s.clone()));
                        colon_found = false;
                        expect_comma = true;
                    }
                    self.advance();
                }
                Token::Number(n) => {
                    err_on_missing_expected_comma(expect_comma, token, self.position())?;
                    err_on_unexpected_value_before_colon(
                        colon_found,
                        &n.to_string(),
                        self.position(),
                    )?;

                    object.insert(key.clone(), JsonValue::Number(*n));
                    colon_found = false;
                    expect_comma = true;

                    self.advance();
                }
                Token::Boolean(b) => {
                    err_on_missing_expected_comma(expect_comma, token, self.position())?;
                    err_on_unexpected_value_before_colon(
                        colon_found,
                        &b.to_string(),
                        self.position(),
                    )?;

                    object.insert(key.clone(), JsonValue::Boolean(*b));
                    colon_found = false;
                    expect_comma = true;

                    self.advance();
                }
                Token::Null => {
                    err_on_missing_expected_comma(expect_comma, token, self.position())?;
                    err_on_unexpected_value_before_colon(colon_found, "null", self.position())?;

                    object.insert(key.clone(), JsonValue::Null);
                    colon_found = false;
                    expect_comma = true;

                    self.advance();
                }
                Token::Colon => {
                    colon_found = true;
                    self.advance();
                }
                Token::Comma => {
                    self.advance(); // Consume comma
                    let token = self.peek().ok_or(unexpected_end_of_input(
                        "string, bool, number or object",
                        self.position(),
                    ))?;

                    err_on_unexpected_comma(expect_comma, "closing brace", self.position())?;
                    err_on_unexpected_closing_token(
                        token,
                        &Token::RightBrace,
                        "string",
                        "}",
                        self.position(),
                    )?;
                    expect_comma = false;
                }
                _ => {
                    return Err(unexpected_token_error(
                        "valid JSON value",
                        &format!("{:?}", token),
                        self.position(),
                    ));
                }
            };
        }

        Err(unexpected_end_of_input("closing brace", self.position()))
    }

    /*
     * Look at current token without advancing
     */
    fn peek(&self) -> Option<&Token> {
        if !self.is_at_end() {
            return self.tokens.get(self.current).map(|(t, _)| t);
        }
        None
    }

    /*
     * Get a token by index (useful to look further ahead)
     */
    fn get_token(&self, index: usize) -> Option<&Token> {
        self.tokens.get(index).map(|(t, _)| t)
    }

    /*
     * Move forward, return previous token
     */
    fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.current);
        self.current += 1;
        token.map(|(t, _)| t)
    }

    /*
     * Byte position of the current token in the original input,
     * or the end of input once the token stream is exhausted
     */
    fn position(&self) -> usize {
        self.tokens
            .get(self.current)
            .map(|(_, position)| *position)
            .unwrap_or(self.input_len)
    }

    /*
     * Check if the input has been consumed
     */
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }
}

fn sp_resolve_escape(byte: u8) -> Option<char> {
    match byte {
        b'n' => Some('\n'),
        b't' => Some('\t'),
        b'r' => Some('\r'),
        b'\\' => Some('\\'),
        b'"' => Some('"'),
        b'/' => Some('/'),
        b'b' => Some('\u{0008}'),
        b'f' => Some('\u{000C}'),
        _ => None,
    }
}

fn sp_parse_unicode_hex(s: &str) -> Option<char> {
    if s.len() != 4 {
        return None;
    }
    u32::from_str_radix(s, 16).ok().and_then(char::from_u32)
}

/*
 * Single-pass parser that scans input bytes and builds JsonValue directly,
 * eliminating the intermediate Vec<Token> and all token-to-value cloning.
 */
struct SinglePassParser<'a> {
    input: &'a str,
    current: usize,
}

impl<'a> SinglePassParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, current: 0 }
    }

    fn peek_byte(&self) -> Option<u8> {
        self.input.as_bytes().get(self.current).copied()
    }

    fn skip_whitespace(&mut self) {
        while let Some(b) = self.peek_byte() {
            match b {
                b' ' | b'\n' | b'\t' | b'\r' => self.current += 1,
                _ => break,
            }
        }
    }

    fn consume_number(&mut self) -> JsonResult<f64> {
        let start = self.current;
        while let Some(b) = self.peek_byte() {
            if !(b.is_ascii_digit()
                || b == b'.'
                || b == b'-'
                || b == b'e'
                || b == b'E'
                || b == b'+')
            {
                break;
            }
            self.current += 1;
        }
        let slice = &self.input[start..self.current];
        slice.parse::<f64>().map_err(|_| JsonError::InvalidNumber {
            value: slice.to_string(),
            position: self.current,
        })
    }

    fn consume_string(&mut self) -> JsonResult<String> {
        let mut start = self.current;
        let mut buf: Option<String> = None;

        loop {
            match self.input.as_bytes().get(self.current) {
                Some(&b'"') => {
                    let tail = &self.input[start..self.current];
                    self.current += 1;
                    return Ok(match buf {
                        None => tail.to_string(),
                        Some(mut s) => {
                            s.push_str(tail);
                            s
                        }
                    });
                }
                Some(&b'\\') => {
                    let s = buf.get_or_insert_with(String::new);
                    s.push_str(&self.input[start..self.current]);
                    self.consume_escape(s)?;
                    start = self.current;
                }
                Some(_) => self.current += 1,
                None => {
                    return Err(JsonError::UnexpectedEndOfInput {
                        expected: "Closing quote".to_string(),
                        position: self.current,
                    });
                }
            }
        }
    }

    fn consume_escape(&mut self, s: &mut String) -> JsonResult<()> {
        self.current += 1;
        let special = self.input.as_bytes().get(self.current).copied().ok_or(
            JsonError::UnexpectedEndOfInput {
                expected: "Special meaning char for escape sequence".to_string(),
                position: self.current,
            },
        )?;
        self.current += 1;
        if special == b'u' {
            let hex_start = self.current;
            if self.current + 4 > self.input.len() {
                return Err(JsonError::InvalidUnicode {
                    sequence: format!("\\u{}", &self.input[hex_start..]),
                    position: self.current,
                });
            }
            let hex_str = &self.input[hex_start..hex_start + 4];
            let ch = sp_parse_unicode_hex(hex_str).ok_or(JsonError::InvalidUnicode {
                sequence: format!("\\u{}", hex_str),
                position: self.current,
            })?;
            s.push(ch);
            self.current += 4;
        } else {
            let ch = sp_resolve_escape(special).ok_or(JsonError::InvalidEscape {
                char: special as char,
                position: self.current,
            })?;
            s.push(ch);
        }
        Ok(())
    }

    fn consume_keyword_value(&mut self) -> JsonResult<JsonValue> {
        let remaining = &self.input.as_bytes()[self.current..];
        if remaining.starts_with(b"true") {
            self.current += 4;
            return Ok(JsonValue::Boolean(true));
        }
        if remaining.starts_with(b"false") {
            self.current += 5;
            return Ok(JsonValue::Boolean(false));
        }
        if remaining.starts_with(b"null") {
            self.current += 4;
            return Ok(JsonValue::Null);
        }
        let found = remaining
            .first()
            .map_or("unknown".to_string(), |&b| (b as char).to_string());
        Err(unexpected_token_error(
            "Valid JSON value",
            &found,
            self.current,
        ))
    }

    fn parse_value(&mut self) -> JsonResult<JsonValue> {
        self.skip_whitespace();
        loop {
            match self.peek_byte() {
                Some(b'{') => return self.parse_object(),
                Some(b'[') => return self.parse_array(),
                Some(b'"') => {
                    self.current += 1;
                    return Ok(JsonValue::String(self.consume_string()?));
                }
                Some(b'0'..=b'9' | b'-') => {
                    return Ok(JsonValue::Number(self.consume_number()?));
                }
                Some(b) if b.is_ascii_alphabetic() => return self.consume_keyword_value(),
                Some(b) if b.is_ascii_punctuation() => {
                    return Err(unexpected_token_error(
                        "Valid JSON value",
                        &(b as char).to_string(),
                        self.current,
                    ));
                }
                Some(_) => {
                    self.current += 1;
                }
                None => {
                    return Err(unexpected_end_of_input("valid JSON value", self.current));
                }
            }
        }
    }

    fn parse_array(&mut self) -> JsonResult<JsonValue> {
        self.current += 1;
        self.skip_whitespace();
        let mut array = Vec::new();

        if self.peek_byte() == Some(b']') {
            self.current += 1;
            return Ok(JsonValue::Array(array));
        }

        loop {
            let value = self.parse_value()?;
            array.push(value);

            self.skip_whitespace();
            match self.peek_byte() {
                Some(b',') => {
                    self.current += 1;
                    self.skip_whitespace();
                    if self.peek_byte() == Some(b']') {
                        return Err(unexpected_token_error(
                            "string, bool, number or object",
                            "]",
                            self.current,
                        ));
                    }
                }
                Some(b']') => {
                    self.current += 1;
                    return Ok(JsonValue::Array(array));
                }
                Some(b) => {
                    return Err(unexpected_token_error(
                        ",",
                        &(b as char).to_string(),
                        self.current,
                    ));
                }
                None => {
                    return Err(unexpected_end_of_input("closing bracket", self.current));
                }
            }
        }
    }

    fn parse_object(&mut self) -> JsonResult<JsonValue> {
        self.current += 1;
        self.skip_whitespace();
        let mut object = HashMap::new();

        if self.peek_byte() == Some(b'}') {
            self.current += 1;
            return Ok(JsonValue::Object(object));
        }

        loop {
            self.skip_whitespace();
            match self.peek_byte() {
                Some(b'"') => self.current += 1,
                Some(b) => {
                    return Err(unexpected_token_error(
                        "string",
                        &(b as char).to_string(),
                        self.current,
                    ));
                }
                None => {
                    return Err(unexpected_end_of_input("string", self.current));
                }
            }
            let key = self.consume_string()?;

            self.skip_whitespace();
            match self.peek_byte() {
                Some(b':') => self.current += 1,
                Some(b) => {
                    return Err(unexpected_token_error(
                        ":",
                        &(b as char).to_string(),
                        self.current,
                    ));
                }
                None => {
                    return Err(unexpected_end_of_input(":", self.current));
                }
            }

            let value = self.parse_value()?;
            object.insert(key, value);

            self.skip_whitespace();
            match self.peek_byte() {
                Some(b',') => {
                    self.current += 1;
                    self.skip_whitespace();
                    if self.peek_byte() == Some(b'}') {
                        return Err(unexpected_token_error("string", "}", self.current));
                    }
                }
                Some(b'}') => {
                    self.current += 1;
                    return Ok(JsonValue::Object(object));
                }
                Some(b) => {
                    return Err(unexpected_token_error(
                        ",",
                        &(b as char).to_string(),
                        self.current,
                    ));
                }
                None => {
                    return Err(unexpected_end_of_input("closing brace", self.current));
                }
            }
        }
    }
}

/// Parses a JSON string and returns the corresponding [`JsonValue`].
///
/// This is the main entry point for parsing JSON. It scans and parses in a single pass.
///
/// # Examples
///
/// ```
/// use rust_json_parser::{parse_json, JsonValue};
///
/// let value = parse_json(r#"{"name": "Alice"}"#)?;
/// assert_eq!(value.get("name"), Some(&JsonValue::String("Alice".to_string())));
///
/// let value = parse_json("[1, 2, 3]")?;
/// assert_eq!(value.as_array().map(|a| a.len()), Some(3));
/// # Ok::<(), rust_json_parser::JsonError>(())
/// ```
///
/// # Errors
///
/// Returns a [`JsonError`] if the input is not valid JSON. This includes
/// lexical errors (invalid characters, malformed strings or numbers) and structural
/// errors (missing commas, unclosed brackets, etc.).
pub fn parse_json(input: &str) -> JsonResult<JsonValue> {
    SinglePassParser::new(input).parse_value()
}

/// Reads a file at the given path and parses its contents as JSON.
///
/// # Examples
///
/// ```no_run
/// use rust_json_parser::parse_json_file;
///
/// let value = parse_json_file("data.json")?;
/// println!("{}", value);
/// # Ok::<(), rust_json_parser::JsonError>(())
/// ```
///
/// # Errors
///
/// Returns [`JsonError::Io`] if the file cannot be read (e.g. not
/// found or permission denied), or any other [`JsonError`] variant if the
/// file contents are not valid JSON.
pub fn parse_json_file(path: &str) -> JsonResult<JsonValue> {
    let contents = fs::read_to_string(path)?;
    parse_json(&contents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::JsonError;

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

    #[test]
    fn test_structural_error_reports_line_column() {
        let input = "{\n  \"key\": 42\n  \"key2\": true\n}";
        // Missing comma after 42: error points at "key2" (line 3, column 3)
        let mut parser = JsonParser::new(input).unwrap();
        let err = parser.parse().unwrap_err();
        assert_eq!(err.line_column(input), Some((3, 3)));
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

    #[test]
    fn test_array_accessor() {
        let value = parse_json("[1, 2, 3]").unwrap();
        let arr = value.as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_array_get_index() {
        let value = parse_json("[10, 20, 30]").unwrap();
        assert_eq!(value.get_index(1), Some(&JsonValue::Number(20.0)));
        assert_eq!(value.get_index(5), None);
    }

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

    #[test]
    fn test_object_accessor() {
        let value = parse_json(r#"{"name": "test"}"#).unwrap();
        let obj = value.as_object().unwrap();
        assert_eq!(obj.len(), 1);
    }

    #[test]
    fn test_object_get() {
        let value = parse_json(r#"{"name": "Alice", "age": 30}"#).unwrap();
        assert_eq!(
            value.get("name"),
            Some(&JsonValue::String("Alice".to_string()))
        );
        assert_eq!(value.get("missing"), None);
    }

    // === Serialization Tests ===

    #[test]
    fn test_display_nested() {
        let value = parse_json(r#"{"arr": [1, 2]}"#).unwrap();
        let output = value.to_string();
        // Object key order may vary, so check components
        assert!(output.contains("\"arr\""));
        assert!(output.contains("[1,2]"));
    }

    #[test]
    fn test_display_nested_array() {
        let value = parse_json(r#"[[[1,2]]]"#).unwrap();
        let output = value.to_string();

        assert_eq!(output, "[[[1,2]]]");
    }

    #[test]
    fn test_display_nested_object() {
        let value = parse_json(r#"{"arr": {"nested": 1, "more": "end"}}"#).unwrap();
        let output = value.to_string();

        assert!(output.contains("\"arr\": {"));
        assert!(output.contains("}}"));
        assert!(output.contains("\"nested\": 1"));
        assert!(output.contains("\"more\": \"end\""));
    }
}
