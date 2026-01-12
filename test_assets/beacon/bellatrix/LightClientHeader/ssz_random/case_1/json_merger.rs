
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str).into());
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Key '{}' already exists, overwriting.", key);
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err("Top-level JSON must be an object".into());
        }
    }

    Ok(Value::Object(merged_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"name": "Alice", "age": 30}"#).unwrap();
        writeln!(file2, r#"{"city": "Berlin", "active": true}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]).unwrap();

        let expected: Value = serde_json::from_str(
            r#"{"name": "Alice", "age": 30, "city": "Berlin", "active": true}"#
        ).unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_overwrite_keys() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"id": 1, "value": "old"}"#).unwrap();
        writeln!(file2, r#"{"id": 2, "extra": "data"}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]).unwrap();

        let id_value = result.get("id").and_then(|v| v.as_i64()).unwrap();
        assert_eq!(id_value, 2);
    }
}
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::Path;

use serde_json::{json, Value};

pub fn merge_json_files(input_paths: &[String], output_path: &str) -> Result<(), String> {
    if input_paths.is_empty() {
        return Err("No input files provided".to_string());
    }

    let mut merged_array = Vec::new();

    for file_path in input_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path));
        }

        let mut file = File::open(path)
            .map_err(|e| format!("Failed to open file {}: {}", file_path, e))?;
        
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read file {}: {}", file_path, e))?;

        let json_value: Value = serde_json::from_str(&contents)
            .map_err(|e| format!("Invalid JSON in file {}: {}", file_path, e))?;

        match json_value {
            Value::Array(arr) => {
                merged_array.extend(arr);
            }
            Value::Object(_) => {
                merged_array.push(json_value);
            }
            _ => {
                return Err(format!("JSON in file {} must be an object or array", file_path));
            }
        }
    }

    let output_json = json!(merged_array);
    
    let mut output_file = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    let pretty_json = serde_json::to_string_pretty(&output_json)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;
    
    output_file.write_all(pretty_json.as_bytes())
        .map_err(|e| format!("Failed to write output file: {}", e))?;

    Ok(())
}

pub fn merge_json_with_deduplication(
    input_paths: &[String], 
    output_path: &str, 
    key_field: &str
) -> Result<usize, String> {
    if input_paths.is_empty() {
        return Err("No input files provided".to_string());
    }

    let mut unique_items = HashMap::new();
    let mut total_processed = 0;

    for file_path in input_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path));
        }

        let file = File::open(path)
            .map_err(|e| format!("Failed to open file {}: {}", file_path, e))?;
        
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)
            .map_err(|e| format!("Invalid JSON in file {}: {}", file_path, e))?;

        let items = match json_value {
            Value::Array(arr) => arr,
            Value::Object(obj) => vec![Value::Object(obj)],
            _ => return Err(format!("JSON in file {} must be an object or array", file_path)),
        };

        for item in items {
            total_processed += 1;
            
            if let Value::Object(map) = &item {
                if let Some(key_value) = map.get(key_field) {
                    let key = key_value.to_string();
                    unique_items.insert(key, item);
                }
            }
        }
    }

    let merged_array: Vec<Value> = unique_items.into_values().collect();
    
    let output_json = json!(merged_array);
    
    let mut output_file = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    let pretty_json = serde_json::to_string_pretty(&output_json)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;
    
    output_file.write_all(pretty_json.as_bytes())
        .map_err(|e| format!("Failed to write output file: {}", e))?;

    Ok(total_processed - merged_array.len())
}