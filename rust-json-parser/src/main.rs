use rust_json_parser::parser::parse_json;
use std::env;
use std::fs;

fn main() {
    let path = env::args().nth(1).expect("Usage: parse_file <path>");
    let contents =
        fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read file: {}", path));

    match parse_json(&contents) {
        Ok(value) => println!("{}", value),
        Err(e) => eprintln!("Parse error: {:?}", e),
    }
}
