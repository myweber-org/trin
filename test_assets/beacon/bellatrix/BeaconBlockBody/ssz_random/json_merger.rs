use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], deduplicate: bool) -> Result<Value, String> {
    let mut merged_array = Vec::new();
    let mut seen_keys = HashSet::new();

    for path in paths {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader
            .read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let json_value: Value = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    if deduplicate {
                        if let Some(id) = item.get("id").and_then(Value::as_str) {
                            if seen_keys.insert(id.to_string()) {
                                merged_array.push(item);
                            }
                        } else {
                            merged_array.push(item);
                        }
                    } else {
                        merged_array.push(item);
                    }
                }
            }
            _ => {
                return Err("Each JSON file must contain a JSON array".to_string());
            }
        }
    }

    Ok(Value::Array(merged_array))
}

pub fn write_merged_json<P: AsRef<Path>>(
    output_path: P,
    value: &Value,
) -> Result<(), String> {
    let file = File::create(output_path).map_err(|e| format!("Failed to create output file: {}", e))?;
    serde_json::to_writer_pretty(file, value).map_err(|e| format!("Failed to write JSON: {}", e))?;
    Ok(())
}