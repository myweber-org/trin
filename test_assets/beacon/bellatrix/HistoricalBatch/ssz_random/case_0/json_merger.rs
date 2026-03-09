
use serde_json::{Value, Map};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <output_file.json> <input1.json> [input2.json ...]", args[0]);
        std::process::exit(1);
    }

    let output_path = &args[1];
    let input_paths = &args[2..];

    let mut merged_map = Map::new();

    for (index, input_path) in input_paths.iter().enumerate() {
        let path = Path::new(input_path);
        if !path.exists() {
            eprintln!("Warning: File '{}' not found, skipping.", input_path);
            continue;
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        let key_name = match path.file_stem() {
            Some(stem) => stem.to_string_lossy().to_string(),
            None => format!("file_{}", index + 1),
        };

        merged_map.insert(key_name, json_value);
    }

    let output_value = Value::Object(merged_map);
    let mut output_file = File::create(output_path)?;
    let json_string = serde_json::to_string_pretty(&output_value)?;
    output_file.write_all(json_string.as_bytes())?;

    println!("Successfully merged {} files into '{}'", input_paths.len(), output_path);
    Ok(())
}