use serde_json::{Map, Value};
use std::collections::HashMap;
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
                    eprintln!("Warning: Duplicate key '{}' found in file '{}'. Overwriting.", key, path_str);
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err(format!("File '{}' does not contain a JSON object at root.", path_str).into());
        }
    }

    Ok(Value::Object(merged_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        let data1 = json!({
            "name": "Alice",
            "age": 30
        });
        let data2 = json!({
            "city": "Berlin",
            "age": 35
        });

        write!(file1, "{}", data1.to_string()).unwrap();
        write!(file2, "{}", data2.to_string()).unwrap();

        let result = merge_json_files(&[file1.path().to_str().unwrap(), file2.path().to_str().unwrap()]);
        assert!(result.is_ok());

        let merged = result.unwrap();
        assert_eq!(merged["name"], "Alice");
        assert_eq!(merged["age"], 35);
        assert_eq!(merged["city"], "Berlin");
    }

    #[test]
    fn test_file_not_found() {
        let result = merge_json_files(&["non_existent.json"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_json_root() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "[1, 2, 3]").unwrap();

        let result = merge_json_files(&[file.path().to_str().unwrap()]);
        assert!(result.is_err());
    }
}