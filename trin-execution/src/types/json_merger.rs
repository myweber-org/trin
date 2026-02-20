
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();
    
    for path in paths {
        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;
        
        if let Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    let existing = merged_map.get(&key).unwrap();
                    
                    if existing.is_object() && value.is_object() {
                        if let (Value::Object(existing_obj), Value::Object(new_obj)) = (existing, &value) {
                            let mut combined_obj = existing_obj.clone();
                            for (sub_key, sub_value) in new_obj {
                                combined_obj.insert(sub_key.clone(), sub_value.clone());
                            }
                            merged_map.insert(key, Value::Object(combined_obj));
                        }
                    } else if existing.is_array() && value.is_array() {
                        if let (Value::Array(existing_arr), Value::Array(new_arr)) = (existing, &value) {
                            let mut combined_arr = existing_arr.clone();
                            combined_arr.extend(new_arr.clone());
                            merged_map.insert(key, Value::Array(combined_arr));
                        }
                    } else {
                        merged_map.insert(key + "_conflict", value);
                    }
                } else {
                    merged_map.insert(key, value);
                }
            }
        }
    }
    
    let merged_json = Value::Object(merged_map);
    let pretty_json = serde_json::to_string_pretty(&merged_json)?;
    fs::write(output_path, pretty_json)?;
    
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
        let output = NamedTempFile::new().unwrap();
        
        fs::write(&file1, r#"{"name": "test", "values": [1, 2]}"#).unwrap();
        fs::write(&file2, r#"{"version": "1.0", "values": [3, 4]}"#).unwrap();
        
        let paths = [file1.path(), file2.path()];
        merge_json_files(&paths, output.path()).unwrap();
        
        let result = fs::read_to_string(output.path()).unwrap();
        assert!(result.contains("\"name\""));
        assert!(result.contains("\"version\""));
        assert!(result.contains("[1,2,3,4]"));
    }
}
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, String> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path_str, e))?;

        let json_value: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path_str, e))?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                if merged_map.contains_key(&key) {
                    return Err(format!("Duplicate key '{}' found in {}", key, path_str));
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err(format!("Top-level JSON must be an object in {}", path_str));
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn merge_json_with_strategy(
    file_paths: &[&str],
    conflict_strategy: ConflictStrategy,
) -> Result<Value, String> {
    let mut accumulator: HashMap<String, Value> = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path_str, e))?;

        let json_value: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path_str, e))?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                match conflict_strategy {
                    ConflictStrategy::Overwrite => {
                        accumulator.insert(key.clone(), value);
                    }
                    ConflictStrategy::Skip => {
                        accumulator.entry(key.clone()).or_insert(value);
                    }
                    ConflictStrategy::Error => {
                        if accumulator.contains_key(&key) {
                            return Err(format!(
                                "Duplicate key '{}' found in {}",
                                key, path_str
                            ));
                        }
                        accumulator.insert(key.clone(), value);
                    }
                }
            }
        } else {
            return Err(format!("Top-level JSON must be an object in {}", path_str));
        }
    }

    let mut map = Map::new();
    for (key, value) in accumulator {
        map.insert(key, value);
    }
    Ok(Value::Object(map))
}

pub enum ConflictStrategy {
    Overwrite,
    Skip,
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_json(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn test_merge_json_files_success() {
        let file1 = create_temp_json(r#"{"a": 1, "b": 2}"#);
        let file2 = create_temp_json(r#"{"c": 3, "d": 4}"#);

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ])
        .unwrap();

        let expected = json!({
            "a": 1,
            "b": 2,
            "c": 3,
            "d": 4
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_with_overwrite_strategy() {
        let file1 = create_temp_json(r#"{"a": 1, "b": 2}"#);
        let file2 = create_temp_json(r#"{"b": 99, "c": 3}"#);

        let result = merge_json_with_strategy(
            &[
                file1.path().to_str().unwrap(),
                file2.path().to_str().unwrap(),
            ],
            ConflictStrategy::Overwrite,
        )
        .unwrap();

        let expected = json!({
            "a": 1,
            "b": 99,
            "c": 3
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_with_skip_strategy() {
        let file1 = create_temp_json(r#"{"a": 1, "b": 2}"#);
        let file2 = create_temp_json(r#"{"b": 99, "c": 3}"#);

        let result = merge_json_with_strategy(
            &[
                file1.path().to_str().unwrap(),
                file2.path().to_str().unwrap(),
            ],
            ConflictStrategy::Skip,
        )
        .unwrap();

        let expected = json!({
            "a": 1,
            "b": 2,
            "c": 3
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_with_error_strategy_fails_on_duplicate() {
        let file1 = create_temp_json(r#"{"a": 1, "b": 2}"#);
        let file2 = create_temp_json(r#"{"b": 99, "c": 3}"#);

        let result = merge_json_with_strategy(
            &[
                file1.path().to_str().unwrap(),
                file2.path().to_str().unwrap(),
            ],
            ConflictStrategy::Error,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Duplicate key"));
    }
}