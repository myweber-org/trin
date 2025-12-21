use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_keys = HashMap::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", file_path);
            continue;
        }

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_data: JsonValue = serde_json::from_str(&contents)?;

        match json_data {
            JsonValue::Array(arr) => {
                for item in arr {
                    if let Some(key) = item.get("id").and_then(|v| v.as_str()) {
                        if seen_keys.contains_key(key) {
                            eprintln!("Duplicate key '{}' found, skipping.", key);
                            continue;
                        }
                        seen_keys.insert(key.to_string(), true);
                    }
                    merged_array.push(item);
                }
            }
            JsonValue::Object(obj) => {
                if let Some(key) = obj.get("id").and_then(|v| v.as_str()) {
                    if seen_keys.contains_key(key) {
                        eprintln!("Duplicate key '{}' found, skipping.", key);
                        continue;
                    }
                    seen_keys.insert(key.to_string(), true);
                }
                merged_array.push(JsonValue::Object(obj));
            }
            _ => {
                eprintln!("Unsupported JSON structure in file {}, skipping.", file_path);
            }
        }
    }

    let output_json = JsonValue::Array(merged_array);
    let mut output_file = File::create(output_path)?;
    write!(output_file, "{}", serde_json::to_string_pretty(&output_json)?)?;

    println!("Successfully merged {} files into {}", file_paths.len(), output_path);
    Ok(())
}