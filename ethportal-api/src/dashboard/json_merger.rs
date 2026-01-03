
use serde_json::{Map, Value};
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file1.json> [file2.json ...]", args[0]);
        process::exit(1);
    }

    let mut merged = Map::new();

    for file_path in &args[1..] {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading file {}: {}", file_path, e);
                process::exit(1);
            }
        };

        let json_data: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Error parsing JSON from {}: {}", file_path, e);
                process::exit(1);
            }
        };

        if let Value::Object(obj) = json_data {
            for (key, value) in obj {
                merged.insert(key, value);
            }
        } else {
            eprintln!("Error: File {} does not contain a JSON object", file_path);
            process::exit(1);
        }
    }

    let output = Value::Object(merged);
    println!("{}", output.to_string());
}