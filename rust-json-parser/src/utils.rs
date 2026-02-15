pub fn resolve_escape_sequence(char: char) -> Option<char> {
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

pub fn escape_json_string(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\t' => result.push_str("\\t"),
            '\r' => result.push_str("\\r"),
            '\u{0008}' => result.push_str("\\b"),
            '\u{000C}' => result.push_str("\\f"),
            _ => result.push(c),
        }
    }
    result
}
