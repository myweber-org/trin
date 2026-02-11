
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

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
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
        writeln!(file2, r#"{"city": "London", "active": true}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]).unwrap();

        let expected = serde_json::json!({
            "name": "Alice",
            "age": 30,
            "city": "London",
            "active": true
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_with_overwrite() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"id": 1, "value": "old"}"#).unwrap();
        writeln!(file2, r#"{"id": 2, "value": "new"}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]).unwrap();

        assert_eq!(result["id"], 2);
        assert_eq!(result["value"], "new");
    }
}use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(serde_json::Value::Object(merged_map.into_iter().collect()))
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

        writeln!(file1, r#"{"a": 1, "b": "test"}"#).unwrap();
        writeln!(file2, r#"{"c": true, "d": [1,2,3]}"#).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("a").unwrap().as_i64().unwrap(), 1);
        assert_eq!(obj.get("b").unwrap().as_str().unwrap(), "test");
        assert_eq!(obj.get("c").unwrap().as_bool().unwrap(), true);
        assert!(obj.get("d").unwrap().is_array());
    }
}
use serde_json::{Map, Value};
use std::collections::HashSet;

pub fn merge_json(base: &mut Value, extension: &Value, conflict_strategy: ConflictStrategy) -> Result<(), String> {
    match (base, extension) {
        (Value::Object(base_map), Value::Object(ext_map)) => {
            merge_objects(base_map, ext_map, conflict_strategy)
        }
        _ => Err("Both values must be JSON objects".to_string()),
    }
}

fn merge_objects(base: &mut Map<String, Value>, extension: &Map<String, Value>, strategy: ConflictStrategy) -> Result<(), String> {
    let base_keys: HashSet<_> = base.keys().collect();
    let ext_keys: HashSet<_> = extension.keys().collect();
    let conflicts: Vec<_> = base_keys.intersection(&ext_keys).collect();

    if !conflicts.is_empty() {
        match strategy {
            ConflictStrategy::PreferBase => return Ok(()),
            ConflictStrategy::PreferExtension => {
                for key in conflicts {
                    base.remove(*key);
                }
            }
            ConflictStrategy::MergeRecursive => {
                for key in conflicts {
                    let base_val = base.get_mut(*key).unwrap();
                    let ext_val = extension.get(*key).unwrap();
                    if let (Value::Object(_), Value::Object(_)) = (base_val, ext_val) {
                        merge_json(base_val, ext_val, strategy.clone())?;
                    } else {
                        base.insert((*key).clone(), ext_val.clone());
                    }
                }
            }
            ConflictStrategy::FailOnConflict => {
                return Err(format!("Conflict detected on keys: {:?}", conflicts));
            }
        }
    }

    for (key, value) in extension {
        if !base.contains_key(key) {
            base.insert(key.clone(), value.clone());
        }
    }

    Ok(())
}

#[derive(Clone, Debug)]
pub enum ConflictStrategy {
    PreferBase,
    PreferExtension,
    MergeRecursive,
    FailOnConflict,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_no_conflict() {
        let mut base = json!({"a": 1});
        let extension = json!({"b": 2});
        merge_json(&mut base, &extension, ConflictStrategy::PreferBase).unwrap();
        assert_eq!(base, json!({"a": 1, "b": 2}));
    }

    #[test]
    fn test_merge_prefer_base() {
        let mut base = json!({"a": 1});
        let extension = json!({"a": 99, "b": 2});
        merge_json(&mut base, &extension, ConflictStrategy::PreferBase).unwrap();
        assert_eq!(base, json!({"a": 1, "b": 2}));
    }

    #[test]
    fn test_merge_prefer_extension() {
        let mut base = json!({"a": 1});
        let extension = json!({"a": 99, "b": 2});
        merge_json(&mut base, &extension, ConflictStrategy::PreferExtension).unwrap();
        assert_eq!(base, json!({"a": 99, "b": 2}));
    }

    #[test]
    fn test_merge_fail_on_conflict() {
        let mut base = json!({"a": 1});
        let extension = json!({"a": 99});
        let result = merge_json(&mut base, &extension, ConflictStrategy::FailOnConflict);
        assert!(result.is_err());
    }
}