
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut result = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            merge_objects(&mut result, obj);
        }
    }

    Ok(Value::Object(result))
}

fn merge_objects(target: &mut Map<String, Value>, source: Map<String, Value>) {
    for (key, source_value) in source {
        match target.get_mut(&key) {
            Some(target_value) => {
                if let (Value::Object(mut target_obj), Value::Object(source_obj)) = (target_value.clone(), source_value.clone()) {
                    merge_objects(&mut target_obj, source_obj);
                    target.insert(key, Value::Object(target_obj));
                } else if target_value != &source_value {
                    let merged_array = match (target_value, &source_value) {
                        (Value::Array(arr1), Value::Array(arr2)) => {
                            let mut combined = arr1.clone();
                            combined.extend(arr2.clone());
                            Value::Array(combined)
                        }
                        _ => Value::Array(vec![target_value.clone(), source_value.clone()]),
                    };
                    target.insert(key, merged_array);
                }
            }
            None => {
                target.insert(key, source_value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() -> Result<(), Box<dyn std::error::Error>> {
        let file1 = NamedTempFile::new()?;
        let file2 = NamedTempFile::new()?;

        fs::write(&file1, r#"{"a": 1, "b": {"c": 2}}"#)?;
        fs::write(&file2, r#"{"b": {"d": 3}, "e": 4}"#)?;

        let merged = merge_json_files(&[file1.path(), file2.path()])?;
        let expected = json!({
            "a": 1,
            "b": {"c": 2, "d": 3},
            "e": 4
        });

        assert_eq!(merged, expected);
        Ok(())
    }
}use serde_json::{Map, Value};
use std::collections::HashSet;

pub fn merge_json(base: &mut Value, update: &Value, overwrite_arrays: bool) {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (key, update_value) in update_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    merge_json(base_value, update_value, overwrite_arrays);
                } else {
                    base_map.insert(key.clone(), update_value.clone());
                }
            }
        }
        (Value::Array(base_arr), Value::Array(update_arr)) if !overwrite_arrays => {
            let mut existing_set = HashSet::new();
            for item in base_arr.iter() {
                if let Some(obj) = item.as_object() {
                    if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                        existing_set.insert(id.to_string());
                    }
                }
            }

            for update_item in update_arr {
                if let Some(obj) = update_item.as_object() {
                    if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                        if existing_set.contains(id) {
                            if let Some(existing) = base_arr
                                .iter_mut()
                                .find(|item| item.get("id").and_then(|v| v.as_str()) == Some(id))
                            {
                                merge_json(existing, update_item, overwrite_arrays);
                            }
                        } else {
                            base_arr.push(update_item.clone());
                        }
                    } else {
                        base_arr.push(update_item.clone());
                    }
                } else {
                    base_arr.push(update_item.clone());
                }
            }
        }
        (base, update) => {
            *base = update.clone();
        }
    }
}

pub fn merge_json_with_config(base: &mut Value, update: &Value, config: &MergeConfig) {
    merge_json(base, update, config.overwrite_arrays);
}

pub struct MergeConfig {
    pub overwrite_arrays: bool,
    pub preserve_null_values: bool,
}

impl Default for MergeConfig {
    fn default() -> Self {
        Self {
            overwrite_arrays: false,
            preserve_null_values: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_merge() {
        let mut base = json!({
            "name": "Alice",
            "age": 30,
            "address": {
                "city": "New York",
                "zip": "10001"
            }
        });

        let update = json!({
            "age": 31,
            "address": {
                "zip": "10002",
                "country": "USA"
            },
            "email": "alice@example.com"
        });

        merge_json(&mut base, &update, false);

        assert_eq!(base["age"], 31);
        assert_eq!(base["address"]["zip"], "10002");
        assert_eq!(base["address"]["country"], "USA");
        assert_eq!(base["email"], "alice@example.com");
    }

    #[test]
    fn test_array_merge_with_ids() {
        let mut base = json!({
            "items": [
                {"id": "1", "name": "First"},
                {"id": "2", "name": "Second"}
            ]
        });

        let update = json!({
            "items": [
                {"id": "2", "name": "Updated Second"},
                {"id": "3", "name": "Third"}
            ]
        });

        merge_json(&mut base, &update, false);

        let items = base["items"].as_array().unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[1]["name"], "Updated Second");
        assert_eq!(items[2]["name"], "Third");
    }
}use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str).into());
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        if let serde_json::Value::Object(map) = json_value {
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
        writeln!(file2, r#"{"city": "Berlin", "age": 31}"#).unwrap();

        let result = merge_json_files(&[file1.path().to_str().unwrap(), file2.path().to_str().unwrap()]);
        assert!(result.is_ok());

        let merged = result.unwrap();
        assert_eq!(merged["name"], "Alice");
        assert_eq!(merged["age"], 31);
        assert_eq!(merged["city"], "Berlin");
    }

    #[test]
    fn test_missing_file() {
        let result = merge_json_files(&["non_existent.json"]);
        assert!(result.is_err());
    }
}