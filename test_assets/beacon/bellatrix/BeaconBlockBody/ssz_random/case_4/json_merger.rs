
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    if paths.is_empty() {
        return Err("No input files provided".to_string());
    }

    let mut merged_map = Map::new();

    for path in paths {
        let content = fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        let json_value: Value = serde_json::from_str(&content).map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                handle_key_conflict(&mut merged_map, key, value);
            }
        } else {
            return Err(format!("Top-level JSON must be an object in {}", path.as_ref().display()));
        }
    }

    Ok(Value::Object(merged_map))
}

fn handle_key_conflict(map: &mut Map<String, Value>, key: String, new_value: Value) {
    match map.get(&key) {
        Some(existing_value) => {
            if existing_value.is_array() && new_value.is_array() {
                if let (Some(existing_arr), Some(new_arr)) = (existing_value.as_array(), new_value.as_array()) {
                    let mut combined = existing_arr.clone();
                    combined.extend(new_arr.clone());
                    map.insert(key, Value::Array(combined));
                }
            } else if existing_value.is_object() && new_value.is_object() {
                if let (Some(existing_obj), Some(new_obj)) = (existing_value.as_object(), new_value.as_object()) {
                    let mut merged_obj = existing_obj.clone();
                    for (nested_key, nested_value) in new_obj {
                        handle_key_conflict(&mut merged_obj, nested_key.clone(), nested_value.clone());
                    }
                    map.insert(key, Value::Object(merged_obj));
                }
            } else {
                map.insert(key + "_merged", new_value);
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
    fn test_merge_basic_objects() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": "test"}"#).unwrap();
        fs::write(&file2, r#"{"c": true, "d": null}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "a": 1,
            "b": "test",
            "c": true,
            "d": null
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_with_array_concatenation() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"items": [1, 2]}"#).unwrap();
        fs::write(&file2, r#"{"items": [3, 4]}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "items": [1, 2, 3, 4]
        });

        assert_eq!(result, expected);
    }
}