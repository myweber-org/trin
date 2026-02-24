
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::Path;

use serde_json::{json, Value};

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), String> {
    let mut merged_array = Vec::new();

    for path in paths {
        let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path.as_ref().display(), e))?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content).map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;

        let json_value: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse JSON from {}: {}", path.as_ref().display(), e))?;

        match json_value {
            Value::Array(arr) => {
                merged_array.extend(arr);
            }
            Value::Object(obj) => {
                merged_array.push(Value::Object(obj));
            }
            _ => {
                return Err(format!("JSON in {} must be an array or object", path.as_ref().display()));
            }
        }
    }

    let output_json = Value::Array(merged_array);
    let output_str = serde_json::to_string_pretty(&output_json)
        .map_err(|e| format!("Failed to serialize merged JSON: {}", e))?;

    fs::write(output_path, output_str).map_err(|e| format!("Failed to write output file: {}", e))?;

    Ok(())
}

pub fn merge_json_with_deduplication<P: AsRef<Path>>(paths: &[P], output_path: P, key_field: &str) -> Result<(), String> {
    let mut unique_map: HashMap<String, Value> = HashMap::new();

    for path in paths {
        let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path.as_ref().display(), e))?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content).map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;

        let json_value: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse JSON from {}: {}", path.as_ref().display(), e))?;

        let items = match json_value {
            Value::Array(arr) => arr,
            _ => return Err(format!("JSON in {} must be an array", path.as_ref().display())),
        };

        for item in items {
            if let Value::Object(map) = item {
                if let Some(key_value) = map.get(key_field) {
                    let key = key_value.to_string();
                    unique_map.insert(key, Value::Object(map));
                }
            }
        }
    }

    let deduplicated_array: Vec<Value> = unique_map.into_values().collect();
    let output_json = Value::Array(deduplicated_array);
    let output_str = serde_json::to_string_pretty(&output_json)
        .map_err(|e| format!("Failed to serialize deduplicated JSON: {}", e))?;

    fs::write(output_path, output_str).map_err(|e| format!("Failed to write output file: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"[{"id": 1}, {"id": 2}]"#).unwrap();
        fs::write(&file2, r#"[{"id": 3}, {"id": 4}]"#).unwrap();

        let paths = [file1.path(), file2.path()];
        let result = merge_json_files(&paths, output_file.path());

        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 4);
    }

    #[test]
    fn test_merge_with_deduplication() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"[{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]"#).unwrap();
        fs::write(&file2, r#"[{"id": 2, "name": "Robert"}, {"id": 3, "name": "Charlie"}]"#).unwrap();

        let paths = [file1.path(), file2.path()];
        let result = merge_json_with_deduplication(&paths, output_file.path(), "id");

        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();
        let array = parsed.as_array().unwrap();
        assert_eq!(array.len(), 3);
    }
}