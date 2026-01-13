use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

use serde_json::{json, Value};

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    let mut merged_array = Vec::new();

    for path in paths {
        let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path.as_ref().display(), e))?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents).map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;

        let json_value: Value = serde_json::from_str(&contents)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        if let Value::Array(arr) = json_value {
            merged_array.extend(arr);
        } else {
            merged_array.push(json_value);
        }
    }

    Ok(Value::Array(merged_array))
}

pub fn merge_json_files_with_key<P: AsRef<Path>>(paths: &[P], key: &str) -> Result<Value, String> {
    let mut merged_map = HashMap::new();

    for path in paths {
        let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path.as_ref().display(), e))?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents).map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;

        let json_value: Value = serde_json::from_str(&contents)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    if let Some(obj) = item.as_object() {
                        if let Some(key_value) = obj.get(key) {
                            if let Some(key_str) = key_value.as_str() {
                                merged_map.insert(key_str.to_string(), item.clone());
                            }
                        }
                    }
                }
            }
            Value::Object(obj) => {
                if let Some(key_value) = obj.get(key) {
                    if let Some(key_str) = key_value.as_str() {
                        merged_map.insert(key_str.to_string(), Value::Object(obj));
                    }
                }
            }
            _ => {}
        }
    }

    let result_array: Vec<Value> = merged_map.into_values().collect();
    Ok(Value::Array(result_array))
}

pub fn write_merged_json<P: AsRef<Path>>(output_path: P, value: &Value) -> Result<(), String> {
    let json_string = serde_json::to_string_pretty(value)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;
    
    fs::write(output_path, json_string)
        .map_err(|e| format!("Failed to write output file: {}", e))?;
    
    Ok(())
}