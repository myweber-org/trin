
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();
    let mut seen_keys = HashSet::new();

    for &file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path).into());
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                if seen_keys.contains(&key) {
                    return Err(format!("Duplicate key '{}' found in files", key).into());
                }
                seen_keys.insert(key.clone());
                merged_map.insert(key, value);
            }
        } else {
            return Err("JSON root is not an object".into());
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

        let data1 = json!({"name": "Alice", "age": 30});
        let data2 = json!({"city": "Berlin", "country": "Germany"});

        write!(file1, "{}", data1.to_string()).unwrap();
        write!(file2, "{}", data2.to_string()).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let expected = json!({
            "name": "Alice",
            "age": 30,
            "city": "Berlin",
            "country": "Germany"
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_duplicate_key_error() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        let data1 = json!({"key": "value1"});
        let data2 = json!({"key": "value2"});

        write!(file1, "{}", data1.to_string()).unwrap();
        write!(file2, "{}", data2.to_string()).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate key"));
    }
}
use serde_json::{Map, Value};
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
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_merge_json_files() {
        let dir = tempdir().unwrap();
        let file1_path = dir.path().join("data1.json");
        let file2_path = dir.path().join("data2.json");

        let mut file1 = File::create(&file1_path).unwrap();
        file1.write_all(b"{\"name\": \"Alice\", \"age\": 30}").unwrap();

        let mut file2 = File::create(&file2_path).unwrap();
        file2.write_all(b"{\"city\": \"Berlin\", \"active\": true}").unwrap();

        let paths = vec![
            file1_path.to_str().unwrap(),
            file2_path.to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "Alice");
        assert_eq!(obj.get("age").unwrap().as_i64().unwrap(), 30);
        assert_eq!(obj.get("city").unwrap().as_str().unwrap(), "Berlin");
        assert!(obj.get("active").unwrap().as_bool().unwrap());
    }
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;
use std::collections::HashSet;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    if paths.is_empty() {
        return Err("No input files provided".to_string());
    }

    let mut merged = Map::new();
    let mut conflict_log = Vec::new();

    for path in paths {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        
        let json: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        if let Value::Object(obj) = json {
            merge_object(&mut merged, obj, &mut conflict_log, path.as_ref());
        } else {
            return Err("Top-level JSON must be an object".to_string());
        }
    }

    if !conflict_log.is_empty() {
        eprintln!("Conflicts detected during merge:");
        for conflict in &conflict_log {
            eprintln!("  - {}", conflict);
        }
    }

    Ok(Value::Object(merged))
}

fn merge_object(base: &mut Map<String, Value>, 
                incoming: Map<String, Value>,
                conflicts: &mut Vec<String>,
                source_path: &Path) {
    for (key, incoming_value) in incoming {
        match base.get_mut(&key) {
            Some(existing_value) => {
                match (existing_value, incoming_value) {
                    (Value::Object(existing_obj), Value::Object(incoming_obj)) => {
                        if let Value::Object(ref mut obj) = existing_value {
                            merge_object(obj, incoming_obj, conflicts, source_path);
                        }
                    }
                    (Value::Array(existing_arr), Value::Array(incoming_arr)) => {
                        let mut combined = HashSet::new();
                        existing_arr.iter().for_each(|v| { combined.insert(v.to_string()); });
                        incoming_arr.iter().for_each(|v| { combined.insert(v.to_string()); });
                        
                        *existing_arr = combined.into_iter()
                            .map(|s| Value::String(s))
                            .collect();
                    }
                    (existing, incoming) if existing == &incoming => {
                        // Identical values, no conflict
                    }
                    _ => {
                        conflicts.push(format!("Key '{}' from {} overwrites existing value", 
                            key, source_path.display()));
                        base.insert(key, incoming_value);
                    }
                }
            }
            None => {
                base.insert(key, incoming_value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_merge() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": {"x": 10}}"#).unwrap();
        fs::write(&file2, r#"{"c": 3, "b": {"y": 20}}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("a").unwrap().as_i64(), Some(1));
        assert_eq!(obj.get("c").unwrap().as_i64(), Some(3));
        
        let b_obj = obj.get("b").unwrap().as_object().unwrap();
        assert_eq!(b_obj.get("x").unwrap().as_i64(), Some(10));
        assert_eq!(b_obj.get("y").unwrap().as_i64(), Some(20));
    }

    #[test]
    fn test_array_merge() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"items": ["a", "b"]}"#).unwrap();
        fs::write(&file2, r#"{"items": ["b", "c"]}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let items = result.get("items").unwrap().as_array().unwrap();
        
        let mut values: Vec<String> = items.iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        values.sort();

        assert_eq!(values, vec!["a", "b", "c"]);
    }
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    if paths.is_empty() {
        return Err("No input files provided".to_string());
    }

    let mut merged = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        
        let json: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        if let Value::Object(obj) = json {
            merge_objects(&mut merged, obj);
        } else {
            return Err(format!("Top-level element must be an object in {}", path.as_ref().display()));
        }
    }

    Ok(Value::Object(merged))
}

fn merge_objects(target: &mut Map<String, Value>, source: Map<String, Value>) {
    for (key, source_value) in source {
        match target.get_mut(&key) {
            Some(target_value) => {
                if let (Value::Object(target_obj), Value::Object(source_obj)) = (target_value, &source_value) {
                    let mut target_map = target_obj.clone();
                    merge_objects(&mut target_map, source_obj.clone());
                    *target_value = Value::Object(target_map);
                } else if target_value != &source_value {
                    *target_value = Value::Array(vec![
                        target_value.clone(),
                        source_value.clone()
                    ]);
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
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": {"x": 10}}"#).unwrap();
        fs::write(&file2, r#"{"b": {"y": 20}, "c": 3}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        
        let expected = json!({
            "a": 1,
            "b": {"x": 10, "y": 20},
            "c": 3
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_conflict_resolution() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"version": "1.0"}"#).unwrap();
        fs::write(&file2, r#"{"version": "2.0"}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        
        let expected = json!({
            "version": ["1.0", "2.0"]
        });

        assert_eq!(result, expected);
    }
}