
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path).into());
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Key '{}' already exists, overwriting with value from {}", key, file_path);
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object at the root".into());
        }
    }

    Ok(serde_json::Value::Object(merged_map))
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
        writeln!(file2, r#"{"city": "London", "age": 31}"#).unwrap();

        let result = merge_json_files(&[file1.path().to_str().unwrap(), file2.path().to_str().unwrap()]);
        assert!(result.is_ok());

        let merged = result.unwrap();
        let obj = merged.as_object().unwrap();

        assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "Alice");
        assert_eq!(obj.get("city").unwrap().as_str().unwrap(), "London");
        assert_eq!(obj.get("age").unwrap().as_u64().unwrap(), 31);
    }

    #[test]
    fn test_file_not_found() {
        let result = merge_json_files(&["non_existent.json"]);
        assert!(result.is_err());
    }
}
use serde_json::{Value, Map};
use std::collections::HashSet;

pub enum ConflictResolution {
    PreferFirst,
    PreferSecond,
    MergeArrays,
    FailOnConflict,
}

pub fn merge_json(
    first: &Value,
    second: &Value,
    strategy: &ConflictResolution,
) -> Result<Value, String> {
    match (first, second) {
        (Value::Object(map1), Value::Object(map2)) => {
            let mut result = Map::new();
            let mut all_keys: HashSet<String> = map1.keys().chain(map2.keys()).cloned().collect();

            for key in all_keys {
                let v1 = map1.get(&key);
                let v2 = map2.get(&key);

                match (v1, v2) {
                    (Some(val1), Some(val2)) => {
                        if val1 == val2 {
                            result.insert(key.clone(), val1.clone());
                        } else {
                            match strategy {
                                ConflictResolution::PreferFirst => {
                                    result.insert(key.clone(), val1.clone());
                                }
                                ConflictResolution::PreferSecond => {
                                    result.insert(key.clone(), val2.clone());
                                }
                                ConflictResolution::MergeArrays => {
                                    if val1.is_array() && val2.is_array() {
                                        let mut merged_array = val1.as_array().unwrap().clone();
                                        merged_array.extend(val2.as_array().unwrap().clone());
                                        result.insert(key.clone(), Value::Array(merged_array));
                                    } else {
                                        return Err(format!(
                                            "Conflict on key '{}': both values are not arrays",
                                            key
                                        ));
                                    }
                                }
                                ConflictResolution::FailOnConflict => {
                                    return Err(format!(
                                        "Conflict on key '{}': values differ",
                                        key
                                    ));
                                }
                            }
                        }
                    }
                    (Some(val), None) => {
                        result.insert(key.clone(), val.clone());
                    }
                    (None, Some(val)) => {
                        result.insert(key.clone(), val.clone());
                    }
                    (None, None) => unreachable!(),
                }
            }
            Ok(Value::Object(result))
        }
        _ => Err("Both inputs must be JSON objects".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_prefer_first() {
        let a = json!({"x": 1, "y": 2});
        let b = json!({"x": 3, "z": 4});
        let result = merge_json(&a, &b, &ConflictResolution::PreferFirst).unwrap();
        assert_eq!(result["x"], 1);
        assert_eq!(result["y"], 2);
        assert_eq!(result["z"], 4);
    }

    #[test]
    fn test_merge_arrays() {
        let a = json!({"items": [1, 2]});
        let b = json!({"items": [3, 4]});
        let result = merge_json(&a, &b, &ConflictResolution::MergeArrays).unwrap();
        assert_eq!(result["items"], json!([1, 2, 3, 4]));
    }
}