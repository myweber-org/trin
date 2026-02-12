
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path).into());
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
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"name": "Alice", "age": 30}"#).unwrap();
        writeln!(file2, r#"{"city": "London", "active": true}"#).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let expected = json!({
            "name": "Alice",
            "age": 30,
            "city": "London",
            "active": true
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_duplicate_key_overwrites() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"id": 1, "value": "first"}"#).unwrap();
        writeln!(file2, r#"{"id": 2, "extra": "data"}"#).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        assert_eq!(result["id"], json!(2));
    }
}
use std::collections::HashMap;
use std::fs;
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[impl AsRef<Path>]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged = HashMap::new();

    for path in file_paths {
        let content = fs::read_to_string(path)?;
        let json_data: JsonValue = serde_json::from_str(&content)?;

        if let JsonValue::Object(map) = json_data {
            for (key, value) in map {
                merged.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain an object".into());
        }
    }

    Ok(JsonValue::Object(merged.into_iter().collect()))
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

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected: JsonValue = serde_json::from_str(
            r#"{"name": "Alice", "age": 30, "city": "Berlin", "active": true}"#
        ).unwrap();

        assert_eq!(result, expected);
    }
}use serde_json::{Map, Value};
use std::collections::HashSet;

pub fn merge_json(base: &mut Value, update: &Value, deep: bool) -> Result<(), String> {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (key, update_value) in update_map {
                if deep {
                    if let Some(base_value) = base_map.get_mut(key) {
                        merge_json(base_value, update_value, deep)?;
                    } else {
                        base_map.insert(key.clone(), update_value.clone());
                    }
                } else {
                    base_map.insert(key.clone(), update_value.clone());
                }
            }
            Ok(())
        }
        (Value::Array(base_arr), Value::Array(update_arr)) => {
            if deep {
                base_arr.extend(update_arr.clone());
                let unique: HashSet<_> = base_arr.drain(..).collect();
                base_arr.extend(unique);
                base_arr.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
            } else {
                *base_arr = update_arr.clone();
            }
            Ok(())
        }
        (base_val, update_val) => {
            if deep && base_val.is_null() {
                *base_val = update_val.clone();
                Ok(())
            } else {
                *base_val = update_val.clone();
                Ok(())
            }
        }
    }
}

pub fn merge_json_with_strategy(
    base: &mut Value,
    update: &Value,
    strategy: MergeStrategy,
) -> Result<(), String> {
    match strategy {
        MergeStrategy::Shallow => merge_json(base, update, false),
        MergeStrategy::Deep => merge_json(base, update, true),
        MergeStrategy::Custom(merger) => merger(base, update),
    }
}

pub enum MergeStrategy {
    Shallow,
    Deep,
    Custom(fn(&mut Value, &Value) -> Result<(), String>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_shallow_merge() {
        let mut base = json!({"a": 1, "b": {"x": 10}});
        let update = json!({"b": {"y": 20}, "c": 3});
        merge_json(&mut base, &update, false).unwrap();
        assert_eq!(base["b"], json!({"y": 20}));
    }

    #[test]
    fn test_deep_merge() {
        let mut base = json!({"a": 1, "b": {"x": 10}});
        let update = json!({"b": {"y": 20}, "c": 3});
        merge_json(&mut base, &update, true).unwrap();
        assert_eq!(base["b"], json!({"x": 10, "y": 20}));
    }
}