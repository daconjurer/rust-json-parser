use std::error::Error;
use std::fmt;

/// Error type representing all possible failures during JSON parsing and serialization.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonError {
    /// A token was found that does not match what the parser expected at this position.
    UnexpectedToken {
        expected: String,
        found: String,
        position: usize,
    },
    /// The input ended before the parser found a required token.
    UnexpectedEndOfInput {
        expected: String,
        position: usize,
    },
    /// A numeric literal could not be parsed as a valid number.
    InvalidNumber {
        value: String,
        position: usize,
    },
    /// An unrecognized escape sequence was encountered inside a string.
    InvalidEscape {
        char: char,
        position: usize,
    },
    /// A `\uXXXX` escape sequence contains an invalid or incomplete hex value.
    InvalidUnicode {
        sequence: String,
        position: usize,
    },
    /// A file system operation failed (e.g. file not found, permission denied).
    Io {
        message: String,
    },
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonError::UnexpectedToken {
                expected,
                found,
                position,
            } => {
                write!(
                    f,
                    "Unexpected token at position {}: expected {}, found {}",
                    position, expected, found,
                )
            }
            JsonError::UnexpectedEndOfInput { expected, position } => {
                write!(
                    f,
                    "Unexpected end of input at position {}: expected {}",
                    position, expected,
                )
            }
            JsonError::InvalidNumber { value, position } => {
                write!(
                    f,
                    "Invalid number at position {}: value {}",
                    position, value,
                )
            }
            JsonError::InvalidEscape { char, position } => {
                write!(f, "Invalid escape at position {}: char {}", position, char,)
            }
            JsonError::InvalidUnicode { sequence, position } => {
                write!(
                    f,
                    "Invalid Unicode sequence at position {}: sequence {}",
                    position, sequence,
                )
            }
            JsonError::Io { message } => write!(f, "IO error: {}", message),
        }
    }
}

impl Error for JsonError {}

impl From<std::io::Error> for JsonError {
    fn from(err: std::io::Error) -> Self {
        JsonError::Io {
            message: err.to_string(),
        }
    }
}

/// Creates an [`JsonError::UnexpectedToken`] error with the given context.
///
/// # Examples
///
/// ```
/// use rust_json_parser::error::unexpected_token_error;
///
/// let err = unexpected_token_error("number", "@", 5);
/// assert_eq!(err.to_string(), "Unexpected token at position 5: expected number, found @");
/// ```
pub fn unexpected_token_error(expected: &str, found: &str, position: usize) -> JsonError {
    JsonError::UnexpectedToken {
        expected: expected.to_string(),
        found: found.to_string(),
        position,
    }
}

/// Creates an [`JsonError::UnexpectedEndOfInput`] error with the given context.
///
/// # Examples
///
/// ```
/// use rust_json_parser::error::unexpected_end_of_input;
///
/// let err = unexpected_end_of_input("closing quote", 10);
/// assert_eq!(err.to_string(), "Unexpected end of input at position 10: expected closing quote");
/// ```
pub fn unexpected_end_of_input(expected: &str, position: usize) -> JsonError {
    JsonError::UnexpectedEndOfInput {
        expected: expected.to_string(),
        position,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = JsonError::UnexpectedToken {
            expected: "number".to_string(),
            found: "@".to_string(),
            position: 5,
        };

        // Error should be Debug-printable
        assert!(format!("{:?}", error).contains("UnexpectedToken"));
    }

    #[test]
    fn test_error_display() {
        let error = JsonError::UnexpectedToken {
            expected: "valid JSON".to_string(),
            found: "@".to_string(),
            position: 0,
        };

        let message = format!("{}", error);
        assert!(message.contains("position 0"));
        assert!(message.contains("valid JSON"));
        assert!(message.contains("@"));
    }

    #[test]
    fn test_error_variants() {
        let token_error = JsonError::UnexpectedToken {
            expected: "number".to_string(),
            found: "x".to_string(),
            position: 3,
        };

        let eof_error = JsonError::UnexpectedEndOfInput {
            expected: "closing quote".to_string(),
            position: 10,
        };

        let num_error = JsonError::InvalidNumber {
            value: "12.34.56".to_string(),
            position: 0,
        };

        // All variants should be Debug-printable
        let _ = format!("{:?}", token_error);
        let _ = format!("{:?}", eof_error);
        let _ = format!("{:?}", num_error);
    }

    #[test]
    fn test_invalid_escape_display() {
        let err = JsonError::InvalidEscape {
            char: 'q',
            position: 5,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("escape"));
        assert!(msg.contains("q"));
    }

    #[test]
    fn test_invalid_unicode_display() {
        let err = JsonError::InvalidUnicode {
            sequence: "00GG".to_string(),
            position: 3,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("unicode") || msg.contains("Unicode"));
    }

    #[test]
    fn test_error_is_std_error() {
        let err = JsonError::InvalidEscape {
            char: 'x',
            position: 0,
        };
        let _: &dyn std::error::Error = &err; // Must implement Error trait
    }
}
