
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde_json::{Value, Map};

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, String> {
    let mut merged_map = Map::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", file_path, e))?;

        let json_value: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", file_path, e))?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                if merged_map.contains_key(&key) {
                    return Err(format!("Duplicate key '{}' found in {}", key, file_path));
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err(format!("Root element in {} is not a JSON object", file_path));
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

        writeln!(file1, r#"{"name": "test", "count": 42}"#).unwrap();
        writeln!(file2, r#"{"enabled": true, "tags": ["a", "b"]}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]);

        assert!(result.is_ok());
        let merged = result.unwrap();
        assert_eq!(merged["name"], "test");
        assert_eq!(merged["count"], 42);
        assert_eq!(merged["enabled"], true);
        assert_eq!(merged["tags"][0], "a");
    }

    #[test]
    fn test_duplicate_key_error() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"key": "first"}"#).unwrap();
        writeln!(file2, r#"{"key": "second"}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("Duplicate key"));
    }
}