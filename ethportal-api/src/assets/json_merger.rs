
use std::collections::HashMap;
use std::fs;
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[impl AsRef<Path>]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged = HashMap::new();

    for path in file_paths {
        let content = fs::read_to_string(path.as_ref())?;
        let json_data: JsonValue = serde_json::from_str(&content)?;

        if let JsonValue::Object(obj) = json_data {
            for (key, value) in obj {
                merged.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    Ok(serde_json::to_value(merged)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": 2}"#).unwrap();
        fs::write(&file2, r#"{"c": 3, "d": 4}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected: JsonValue = serde_json::from_str(r#"{"a":1,"b":2,"c":3,"d":4}"#).unwrap();

        assert_eq!(result, expected);
    }
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut result = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if result.contains_key(&key) {
                    let existing = result.get(&key).unwrap();
                    result.insert(key, resolve_conflict(existing, &value));
                } else {
                    result.insert(key, value);
                }
            }
        }
    }

    Ok(Value::Object(result))
}

fn resolve_conflict(a: &Value, b: &Value) -> Value {
    match (a, b) {
        (Value::Array(arr_a), Value::Array(arr_b)) => {
            let mut combined = arr_a.clone();
            combined.extend(arr_b.clone());
            Value::Array(combined)
        }
        (Value::Object(obj_a), Value::Object(obj_b)) => {
            let mut merged = obj_a.clone();
            for (key, val_b) in obj_b {
                if merged.contains_key(key) {
                    let val_a = merged.get(key).unwrap();
                    merged.insert(key.clone(), resolve_conflict(val_a, val_b));
                } else {
                    merged.insert(key.clone(), val_b.clone());
                }
            }
            Value::Object(merged)
        }
        _ => b.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_objects() {
        let a = json!({"name": "Alice", "age": 30});
        let b = json!({"name": "Bob", "city": "London"});
        
        let merged = resolve_conflict(&a, &b);
        assert_eq!(merged["name"], "Bob");
        assert_eq!(merged["age"], 30);
        assert_eq!(merged["city"], "London");
    }

    #[test]
    fn test_merge_arrays() {
        let a = json!([1, 2, 3]);
        let b = json!([4, 5]);
        
        let merged = resolve_conflict(&a, &b);
        assert_eq!(merged, json!([1, 2, 3, 4, 5]));
    }
}use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
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

        let json1 = json!({
            "name": "Alice",
            "age": 30
        });

        let json2 = json!({
            "city": "Berlin",
            "active": true
        });

        write!(file1, "{}", json1.to_string()).unwrap();
        write!(file2, "{}", json2.to_string()).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let expected = json!({
            "name": "Alice",
            "age": 30,
            "city": "Berlin",
            "active": true
        });

        assert_eq!(result, expected);
    }
}