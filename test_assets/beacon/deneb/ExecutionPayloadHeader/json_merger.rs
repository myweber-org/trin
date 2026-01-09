use std::collections::HashMap;
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

pub enum MergeStrategy {
    PreferFirst,
    PreferSecond,
    MergeObjects,
    MergeArrays,
    FailOnConflict,
}

pub fn merge_json(a: &Value, b: &Value, strategy: &MergeStrategy) -> Result<Value, String> {
    match (a, b) {
        (Value::Object(map_a), Value::Object(map_b)) => merge_objects(map_a, map_b, strategy),
        (Value::Array(arr_a), Value::Array(arr_b)) => merge_arrays(arr_a, arr_b, strategy),
        _ => handle_scalar_conflict(a, b, strategy),
    }
}

fn merge_objects(
    a: &Map<String, Value>,
    b: &Map<String, Value>,
    strategy: &MergeStrategy,
) -> Result<Value, String> {
    let mut result = Map::new();
    let keys_a: HashSet<_> = a.keys().collect();
    let keys_b: HashSet<_> = b.keys().collect();
    
    for key in keys_a.union(&keys_b) {
        let key_str = (*key).clone();
        match (a.get(key_str.as_str()), b.get(key_str.as_str())) {
            (Some(val_a), Some(val_b)) => {
                let merged = merge_json(val_a, val_b, strategy)?;
                result.insert(key_str, merged);
            }
            (Some(val), None) | (None, Some(val)) => {
                result.insert(key_str, val.clone());
            }
            _ => unreachable!(),
        }
    }
    
    Ok(Value::Object(result))
}

fn merge_arrays(
    a: &[Value],
    b: &[Value],
    strategy: &MergeStrategy,
) -> Result<Value, String> {
    match strategy {
        MergeStrategy::MergeArrays => {
            let mut merged = Vec::with_capacity(a.len() + b.len());
            merged.extend_from_slice(a);
            merged.extend_from_slice(b);
            Ok(Value::Array(merged))
        }
        MergeStrategy::PreferFirst => Ok(Value::Array(a.to_vec())),
        MergeStrategy::PreferSecond => Ok(Value::Array(b.to_vec())),
        _ => Err("Cannot merge arrays with current strategy".to_string()),
    }
}

fn handle_scalar_conflict(a: &Value, b: &Value, strategy: &MergeStrategy) -> Result<Value, String> {
    if a == b {
        return Ok(a.clone());
    }
    
    match strategy {
        MergeStrategy::PreferFirst => Ok(a.clone()),
        MergeStrategy::PreferSecond => Ok(b.clone()),
        MergeStrategy::FailOnConflict => Err(format!("Conflict between values: {} and {}", a, b)),
        _ => Err("Cannot merge scalar values with object/array strategy".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_objects_prefer_first() {
        let a = json!({"x": 1, "y": 2});
        let b = json!({"x": 3, "z": 4});
        let result = merge_json(&a, &b, &MergeStrategy::PreferFirst).unwrap();
        assert_eq!(result["x"], 1);
        assert_eq!(result["y"], 2);
        assert_eq!(result["z"], 4);
    }

    #[test]
    fn test_merge_arrays() {
        let a = json!([1, 2]);
        let b = json!([3, 4]);
        let result = merge_json(&a, &b, &MergeStrategy::MergeArrays).unwrap();
        assert_eq!(result, json!([1, 2, 3, 4]));
    }
}