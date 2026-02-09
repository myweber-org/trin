
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use serde_json::{Map, Value};

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, String> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str));
        }

        let file = File::open(path).map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents).map_err(|e| e.to_string())?;

        let json_value: Value = serde_json::from_str(&contents).map_err(|e| e.to_string())?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    return Err(format!("Duplicate key found: {}", key));
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err("JSON root is not an object".to_string());
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn merge_json_with_strategy(
    file_paths: &[&str],
    conflict_strategy: ConflictStrategy,
) -> Result<Value, String> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str));
        }

        let file = File::open(path).map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents).map_err(|e| e.to_string())?;

        let json_value: Value = serde_json::from_str(&contents).map_err(|e| e.to_string())?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                match conflict_strategy {
                    ConflictStrategy::Error => {
                        if merged_map.contains_key(&key) {
                            return Err(format!("Duplicate key found: {}", key));
                        }
                        merged_map.insert(key, value);
                    }
                    ConflictStrategy::Overwrite => {
                        merged_map.insert(key, value);
                    }
                    ConflictStrategy::Merge => {
                        if let Some(existing) = merged_map.get(&key) {
                            if existing.is_object() && value.is_object() {
                                let merged = merge_json_objects(existing, &value)?;
                                merged_map.insert(key, merged);
                            } else {
                                merged_map.insert(key, value);
                            }
                        } else {
                            merged_map.insert(key, value);
                        }
                    }
                }
            }
        } else {
            return Err("JSON root is not an object".to_string());
        }
    }

    Ok(Value::Object(merged_map))
}

fn merge_json_objects(a: &Value, b: &Value) -> Result<Value, String> {
    let mut result_map = Map::new();

    if let Value::Object(map_a) = a {
        for (key, val) in map_a {
            result_map.insert(key.clone(), val.clone());
        }
    }

    if let Value::Object(map_b) = b {
        for (key, val) in map_b {
            if result_map.contains_key(&key) {
                let existing = result_map.get(&key).unwrap();
                if existing.is_object() && val.is_object() {
                    let merged = merge_json_objects(existing, val)?;
                    result_map.insert(key.clone(), merged);
                } else {
                    result_map.insert(key.clone(), val.clone());
                }
            } else {
                result_map.insert(key.clone(), val.clone());
            }
        }
    }

    Ok(Value::Object(result_map))
}

pub enum ConflictStrategy {
    Error,
    Overwrite,
    Merge,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_json(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn test_merge_json_files() {
        let file1 = create_temp_json(r#"{"a": 1, "b": 2}"#);
        let file2 = create_temp_json(r#"{"c": 3, "d": 4}"#);

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ])
        .unwrap();

        let expected: Value = serde_json::from_str(r#"{"a":1,"b":2,"c":3,"d":4}"#).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_with_conflict_error() {
        let file1 = create_temp_json(r#"{"a": 1}"#);
        let file2 = create_temp_json(r#"{"a": 2}"#);

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Duplicate key"));
    }

    #[test]
    fn test_merge_with_overwrite_strategy() {
        let file1 = create_temp_json(r#"{"a": 1}"#);
        let file2 = create_temp_json(r#"{"a": 2}"#);

        let result = merge_json_with_strategy(
            &[
                file1.path().to_str().unwrap(),
                file2.path().to_str().unwrap(),
            ],
            ConflictStrategy::Overwrite,
        )
        .unwrap();

        let expected: Value = serde_json::from_str(r#"{"a":2}"#).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_with_merge_strategy() {
        let file1 = create_temp_json(r#"{"a": {"b": 1}}"#);
        let file2 = create_temp_json(r#"{"a": {"c": 2}}"#);

        let result = merge_json_with_strategy(
            &[
                file1.path().to_str().unwrap(),
                file2.path().to_str().unwrap(),
            ],
            ConflictStrategy::Merge,
        )
        .unwrap();

        let expected: Value = serde_json::from_str(r#"{"a": {"b":1,"c":2}}"#).unwrap();
        assert_eq!(result, expected);
    }
}