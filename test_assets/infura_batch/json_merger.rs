
use serde_json::{Map, Value};
use std::env;
use std::fs::File;
use std::io::Read;
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

        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

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
    let output_str = serde_json::to_string_pretty(&output)?;
    println!("{}", output_str);

    Ok(())
}