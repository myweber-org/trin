
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

use serde_json::{Value, json};

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    let mut merged_array = Vec::new();

    for path in paths {
        let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path.as_ref().display(), e))?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents).map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;

        let json_value: Value = serde_json::from_str(&contents)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        match json_value {
            Value::Array(arr) => merged_array.extend(arr),
            Value::Object(obj) => merged_array.push(Value::Object(obj)),
            _ => return Err(format!("JSON in {} must be an object or array", path.as_ref().display()))
        }
    }

    Ok(Value::Array(merged_array))
}

pub fn merge_with_key_deduplication<P: AsRef<Path>>(paths: &[P], key_field: &str) -> Result<Value, String> {
    let mut unique_map: HashMap<String, Value> = HashMap::new();

    for path in paths {
        let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path.as_ref().display(), e))?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents).map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;

        let json_value: Value = serde_json::from_str(&contents)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        let items = match json_value {
            Value::Array(arr) => arr,
            Value::Object(obj) => vec![Value::Object(obj)],
            _ => return Err(format!("JSON in {} must be an object or array", path.as_ref().display()))
        };

        for item in items {
            if let Value::Object(map) = item {
                if let Some(key_value) = map.get(key_field) {
                    if let Some(key_str) = key_value.as_str() {
                        unique_map.insert(key_str.to_string(), Value::Object(map));
                    } else {
                        return Err(format!("Key field '{}' must be a string in {}", key_field, path.as_ref().display()));
                    }
                } else {
                    return Err(format!("Missing key field '{}' in {}", key_field, path.as_ref().display()));
                }
            }
        }
    }

    let deduplicated: Vec<Value> = unique_map.into_values().collect();
    Ok(Value::Array(deduplicated))
}

pub fn write_merged_json<P: AsRef<Path>>(output_path: P, value: &Value) -> Result<(), String> {
    let json_string = serde_json::to_string_pretty(value)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;
    
    fs::write(&output_path, json_string)
        .map_err(|e| format!("Failed to write to {}: {}", output_path.as_ref().display(), e))?;
    
    Ok(())
}