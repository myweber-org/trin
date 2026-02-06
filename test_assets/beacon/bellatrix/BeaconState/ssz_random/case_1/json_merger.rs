use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Vec<serde_json::Value>, String> {
    let mut merged_array = Vec::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str));
        }

        let file = File::open(path).map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents).map_err(|e| e.to_string())?;

        let json_value: serde_json::Value =
            serde_json::from_str(&contents).map_err(|e| e.to_string())?;

        match json_value {
            serde_json::Value::Array(arr) => {
                merged_array.extend(arr);
            }
            serde_json::Value::Object(obj) => {
                merged_array.push(serde_json::Value::Object(obj));
            }
            _ => {
                return Err(format!("Unsupported JSON structure in file: {}", path_str));
            }
        }
    }

    Ok(merged_array)
}

pub fn deduplicate_json_array(array: Vec<serde_json::Value>, key: &str) -> Vec<serde_json::Value> {
    let mut seen = HashMap::new();
    let mut deduped = Vec::new();

    for value in array {
        if let Some(obj) = value.as_object() {
            if let Some(id_value) = obj.get(key) {
                let id = id_value.to_string();
                if !seen.contains_key(&id) {
                    seen.insert(id.clone(), true);
                    deduped.push(value);
                }
            } else {
                deduped.push(value);
            }
        } else {
            deduped.push(value);
        }
    }

    deduped
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                merge_value(&mut merged, key, value);
            }
        }
    }

    Ok(Value::Object(merged))
}

fn merge_value(map: &mut Map<String, Value>, key: String, new_value: Value) {
    match map.get_mut(&key) {
        Some(existing) => {
            if let (Value::Object(ref mut existing_obj), Value::Object(new_obj)) = (existing, &new_value) {
                for (nested_key, nested_value) in new_obj {
                    merge_value(existing_obj, nested_key.clone(), nested_value.clone());
                }
            } else if existing != &new_value {
                let conflict_key = format!("{}_conflict", key);
                let conflict_array = match map.get_mut(&conflict_key) {
                    Some(Value::Array(arr)) => arr,
                    _ => {
                        let arr = vec![existing.clone()];
                        map.insert(conflict_key.clone(), Value::Array(arr));
                        map.get_mut(&conflict_key).unwrap().as_array_mut().unwrap()
                    }
                };
                conflict_array.push(new_value);
            }
        }
        None => {
            map.insert(key, new_value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_basic() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": {"c": 2}}"#).unwrap();
        fs::write(&file2, r#"{"b": {"d": 3}, "e": 4}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "a": 1,
            "b": {"c": 2, "d": 3},
            "e": 4
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_conflict() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"version": "1.0.0"}"#).unwrap();
        fs::write(&file2, r#"{"version": "2.0.0"}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let version_conflict = result.get("version_conflict").unwrap().as_array().unwrap();

        assert!(version_conflict.contains(&json!("1.0.0")));
        assert!(version_conflict.contains(&json!("2.0.0")));
    }
}