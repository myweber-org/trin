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

        if let Some(obj) = json_value.as_object() {
            for (key, value) in obj {
                merged_map.insert(key.clone(), value.clone());
            }
        }
    }

    Ok(serde_json::Value::Object(
        merged_map.into_iter().collect()
    ))
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
        writeln!(file2, r#"{"city": "Berlin", "age": 35}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]).unwrap();

        assert_eq!(result["name"], "Alice");
        assert_eq!(result["city"], "Berlin");
        assert_eq!(result["age"], 35);
    }
}use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain an object at the root".into());
        }
    }

    let merged_json = Value::Object(merged_map);
    let json_string = serde_json::to_string_pretty(&merged_json)?;
    fs::write(output_path, json_string)?;

    Ok(())
}
use serde_json::{Map, Value};
use std::collections::HashSet;

pub enum ConflictResolution {
    PreferFirst,
    PreferSecond,
    MergeArrays,
    FailOnConflict,
}

pub fn merge_json(
    first: &Map<String, Value>,
    second: &Map<String, Value>,
    strategy: ConflictResolution,
) -> Result<Map<String, Value>, String> {
    let mut result = first.clone();
    
    for (key, second_value) in second {
        match result.get(key) {
            Some(first_value) => {
                let merged_value = match (first_value, second_value) {
                    (Value::Object(first_obj), Value::Object(second_obj)) => {
                        let first_map = first_obj.as_object().unwrap();
                        let second_map = second_obj.as_object().unwrap();
                        Value::Object(merge_json(first_map, second_map, strategy.clone())?)
                    }
                    (Value::Array(first_arr), Value::Array(second_arr)) => {
                        match strategy {
                            ConflictResolution::MergeArrays => {
                                let mut combined = first_arr.clone();
                                combined.extend(second_arr.clone());
                                Value::Array(combined)
                            }
                            _ => handle_conflict(first_value, second_value, &strategy, key)?,
                        }
                    }
                    _ => handle_conflict(first_value, second_value, &strategy, key)?,
                };
                result.insert(key.clone(), merged_value);
            }
            None => {
                result.insert(key.clone(), second_value.clone());
            }
        }
    }
    
    Ok(result)
}

fn handle_conflict(
    first: &Value,
    second: &Value,
    strategy: &ConflictResolution,
    key: &str,
) -> Result<Value, String> {
    match strategy {
        ConflictResolution::PreferFirst => Ok(first.clone()),
        ConflictResolution::PreferSecond => Ok(second.clone()),
        ConflictResolution::FailOnConflict => {
            Err(format!("Conflict detected for key '{}'", key))
        }
        ConflictResolution::MergeArrays => {
            Err("MergeArrays strategy only applicable to arrays".to_string())
        }
    }
}

pub fn find_common_keys(first: &Map<String, Value>, second: &Map<String, Value>) -> HashSet<String> {
    let first_keys: HashSet<_> = first.keys().cloned().collect();
    let second_keys: HashSet<_> = second.keys().cloned().collect();
    first_keys.intersection(&second_keys).cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_objects() {
        let mut first = Map::new();
        first.insert("a".to_string(), json!(1));
        first.insert("b".to_string(), json!({"nested": true}));
        
        let mut second = Map::new();
        second.insert("b".to_string(), json!({"extra": 42}));
        second.insert("c".to_string(), json!("test"));
        
        let merged = merge_json(&first, &second, ConflictResolution::PreferFirst).unwrap();
        
        assert_eq!(merged.get("a").unwrap(), &json!(1));
        assert_eq!(merged.get("c").unwrap(), &json!("test"));
        let b_obj = merged.get("b").unwrap().as_object().unwrap();
        assert!(b_obj.get("nested").is_some());
        assert!(b_obj.get("extra").is_some());
    }

    #[test]
    fn test_array_merge() {
        let mut first = Map::new();
        first.insert("arr".to_string(), json!([1, 2]));
        
        let mut second = Map::new();
        second.insert("arr".to_string(), json!([3, 4]));
        
        let merged = merge_json(&first, &second, ConflictResolution::MergeArrays).unwrap();
        let arr = merged.get("arr").unwrap().as_array().unwrap();
        assert_eq!(arr, &vec![json!(1), json!(2), json!(3), json!(4)]);
    }
}