use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in input_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", path_str);
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            eprintln!("Warning: File {} does not contain a JSON object, skipping.", path_str);
        }
    }

    let merged_json = Value::Object(merged_map);
    let json_string = serde_json::to_string_pretty(&merged_json)?;
    fs::write(output_path, json_string)?;

    Ok(())
}
use serde_json::{Map, Value};
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file1.json> [file2.json ...]", args[0]);
        std::process::exit(1);
    }

    let mut merged = Map::new();

    for file_path in &args[1..] {
        let path = Path::new(file_path);
        if !path.exists() {
            eprintln!("Warning: File '{}' not found, skipping.", file_path);
            continue;
        }

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: Value = serde_json::from_str(&contents)?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                merged.insert(key, value);
            }
        } else {
            eprintln!("Warning: '{}' does not contain a JSON object, skipping.", file_path);
        }
    }

    let output = Value::Object(merged);
    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}