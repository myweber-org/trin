
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[impl AsRef<Path>]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged = HashMap::new();

    for path in file_paths {
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_data: JsonValue = serde_json::from_str(&contents)?;

        if let JsonValue::Object(map) = json_data {
            for (key, value) in map {
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"name": "Alice", "age": 30}"#).unwrap();
        writeln!(file2, r#"{"city": "Berlin", "active": true}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("name").unwrap(), "Alice");
        assert_eq!(obj.get("age").unwrap(), 30);
        assert_eq!(obj.get("city").unwrap(), "Berlin");
        assert_eq!(obj.get("active").unwrap(), true);
    }
}use serde_json::{Map, Value};
use std::collections::HashSet;

pub fn merge_json(base: &mut Value, other: &Value, overwrite_arrays: bool) {
    match (base, other) {
        (Value::Object(base_map), Value::Object(other_map)) => {
            for (key, other_value) in other_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    merge_json(base_value, other_value, overwrite_arrays);
                } else {
                    base_map.insert(key.clone(), other_value.clone());
                }
            }
        }
        (Value::Array(base_arr), Value::Array(other_arr)) if !overwrite_arrays => {
            let mut seen = HashSet::new();
            for item in base_arr.iter() {
                if let Some(s) = item.as_str() {
                    seen.insert(s.to_string());
                }
            }
            for item in other_arr {
                if let Some(s) = item.as_str() {
                    if !seen.contains(s) {
                        base_arr.push(Value::String(s.to_string()));
                    }
                } else {
                    base_arr.push(item.clone());
                }
            }
        }
        (base, other) => {
            *base = other.clone();
        }
    }
}

pub fn merge_json_with_strategy(
    base: &str,
    other: &str,
    overwrite_arrays: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut base_value: Value = serde_json::from_str(base)?;
    let other_value: Value = serde_json::from_str(other)?;
    
    merge_json(&mut base_value, &other_value, overwrite_arrays);
    
    Ok(serde_json::to_string_pretty(&base_value)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_merge() {
        let base = r#"{"a": 1, "b": {"c": 2}}"#;
        let other = r#"{"b": {"d": 3}, "e": 4}"#;
        let result = merge_json_with_strategy(base, other, false).unwrap();
        let expected = r#"{
  "a": 1,
  "b": {
    "c": 2,
    "d": 3
  },
  "e": 4
}"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_array_merge() {
        let base = r#"{"items": ["apple", "banana"]}"#;
        let other = r#"{"items": ["banana", "cherry"]}"#;
        let result = merge_json_with_strategy(base, other, false).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        let items = parsed["items"].as_array().unwrap();
        assert_eq!(items.len(), 3);
        assert!(items.contains(&Value::String("apple".to_string())));
        assert!(items.contains(&Value::String("banana".to_string())));
        assert!(items.contains(&Value::String("cherry".to_string())));
    }
}