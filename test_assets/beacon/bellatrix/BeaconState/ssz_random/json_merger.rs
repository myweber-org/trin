use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

type JsonValue = serde_json::Value;
type JsonMap = HashMap<String, JsonValue>;

pub fn merge_json_files(file_paths: &[&str]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_map = JsonMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str).into());
        }

        let mut file_content = String::new();
        let mut file = File::open(path)?;
        file.read_to_string(&mut file_content)?;

        let json_val: JsonValue = serde_json::from_str(&file_content)?;

        if let JsonValue::Object(map) = json_val {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Key '{}' already exists, overwriting.", key);
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err("Top-level JSON value must be an object".into());
        }
    }

    Ok(JsonValue::Object(
        merged_map
            .into_iter()
            .map(|(k, v)| (k, v))
            .collect(),
    ))
}

pub fn write_merged_json(output_path: &str, json_value: &JsonValue) -> Result<(), Box<dyn std::error::Error>> {
    let json_string = serde_json::to_string_pretty(json_value)?;
    fs::write(output_path, json_string)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_basic() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(file1.path(), r#"{"a": 1, "b": "test"}"#).unwrap();
        fs::write(file2.path(), r#"{"c": true, "d": [1,2,3]}"#).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];
        let merged = merge_json_files(&paths).unwrap();

        assert_eq!(merged["a"], 1);
        assert_eq!(merged["b"], "test");
        assert_eq!(merged["c"], true);
        assert!(merged["d"].is_array());
    }

    #[test]
    fn test_overwrite_key() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(file1.path(), r#"{"key": "first"}"#).unwrap();
        fs::write(file2.path(), r#"{"key": "second"}"#).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];
        let merged = merge_json_files(&paths).unwrap();

        assert_eq!(merged["key"], "second");
    }
}