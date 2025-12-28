use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

use serde_json::{Value, json};

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
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        let json_value: Value = serde_json::from_str(&content)?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    if let Some(obj) = item.as_object() {
                        if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                            if seen_keys.contains_key(id) {
                                eprintln!("Duplicate key '{}' found in {}, skipping.", id, file_path);
                                continue;
                            }
                            seen_keys.insert(id.to_string(), ());
                        }
                    }
                    merged_array.push(item);
                }
            }
            Value::Object(obj) => {
                if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                    if seen_keys.contains_key(id) {
                        eprintln!("Duplicate key '{}' found in {}, skipping.", id, file_path);
                        continue;
                    }
                    seen_keys.insert(id.to_string(), ());
                }
                merged_array.push(json!(obj));
            }
            _ => {
                eprintln!("Warning: {} does not contain a JSON object or array, skipping.", file_path);
            }
        }
    }

    let output_json = json!(merged_array);
    let output_content = serde_json::to_string_pretty(&output_json)?;
    fs::write(output_path, output_content)?;

    println!("Successfully merged {} files into {}", file_paths.len(), output_path);
    Ok(())
}