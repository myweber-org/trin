
use serde_json::{Value, Map};
use std::fs;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <output_file.json> <input1.json> [input2.json ...]", args[0]);
        process::exit(1);
    }

    let output_path = &args[1];
    let input_paths = &args[2..];

    let mut merged_map = Map::new();

    for (index, path) in input_paths.iter().enumerate() {
        let file_contents = match fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(e) => {
                eprintln!("Failed to read file '{}': {}", path, e);
                process::exit(1);
            }
        };

        let json_value: Value = match serde_json::from_str(&file_contents) {
            Ok(value) => value,
            Err(e) => {
                eprintln!("Failed to parse JSON from '{}': {}", path, e);
                process::exit(1);
            }
        };

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Key '{}' from file '{}' overwrites previous value.", key, path);
                }
                merged_map.insert(key, value);
            }
        } else {
            let default_key = format!("file_{}", index + 1);
            merged_map.insert(default_key, json_value);
        }
    }

    let merged_value = Value::Object(merged_map);
    let json_string = match serde_json::to_string_pretty(&merged_value) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to serialize merged JSON: {}", e);
            process::exit(1);
        }
    };

    if let Err(e) = fs::write(output_path, json_string) {
        eprintln!("Failed to write output file '{}': {}", output_path, e);
        process::exit(1);
    }

    println!("Successfully merged {} JSON files into '{}'", input_paths.len(), output_path);
}