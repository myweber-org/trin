
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;

use serde_json::{Map, Value};

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, String> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path_str, e))?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader
            .read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read {}: {}", path_str, e))?;

        let json_value: Value = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse JSON from {}: {}", path_str, e))?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    return Err(format!("Duplicate key '{}' found in {}", key, path_str));
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err(format!("Top-level JSON in {} is not an object", path_str));
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn write_merged_json(output_path: &str, value: &Value) -> Result<(), String> {
    let json_string = serde_json::to_string_pretty(value)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

    let mut file = File::create(output_path)
        .map_err(|e| format!("Failed to create output file {}: {}", output_path, e))?;

    file.write_all(json_string.as_bytes())
        .map_err(|e| format!("Failed to write to {}: {}", output_path, e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1_content = r#"{"name": "Alice", "age": 30}"#;
        let file2_content = r#"{"city": "London", "country": "UK"}"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(file1.path(), file1_content).unwrap();
        fs::write(file2.path(), file2_content).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "Alice");
        assert_eq!(obj.get("age").unwrap().as_u64().unwrap(), 30);
        assert_eq!(obj.get("city").unwrap().as_str().unwrap(), "London");
        assert_eq!(obj.get("country").unwrap().as_str().unwrap(), "UK");
        assert_eq!(obj.len(), 4);
    }

    #[test]
    fn test_duplicate_key_error() {
        let file1_content = r#"{"key": "value1"}"#;
        let file2_content = r#"{"key": "value2"}"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(file1.path(), file1_content).unwrap();
        fs::write(file2.path(), file2_content).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Duplicate key"));
    }
}