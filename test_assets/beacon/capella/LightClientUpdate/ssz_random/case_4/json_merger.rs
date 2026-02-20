use serde_json::{Map, Value};
use std::collections::HashMap;
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
                    eprintln!("Warning: Duplicate key '{}' found in file '{}'. Overwriting.", key, path_str);
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err(format!("Root element in '{}' is not a JSON object", path_str).into());
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn merge_json_with_strategy(
    file_paths: &[&str],
    conflict_strategy: ConflictStrategy,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut accumulator: HashMap<String, Value> = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str).into());
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                match conflict_strategy {
                    ConflictStrategy::Overwrite => {
                        accumulator.insert(key, value);
                    }
                    ConflictStrategy::Skip => {
                        accumulator.entry(key).or_insert(value);
                    }
                    ConflictStrategy::MergeObjects => {
                        if let Some(existing) = accumulator.get_mut(&key) {
                            if let (Value::Object(existing_obj), Value::Object(new_obj)) = (existing, &value) {
                                let mut merged = existing_obj.clone();
                                for (k, v) in new_obj {
                                    merged.insert(k.clone(), v.clone());
                                }
                                *existing = Value::Object(merged);
                            } else {
                                eprintln!("Warning: Key '{}' conflict - cannot merge non-objects. Skipping.", key);
                            }
                        } else {
                            accumulator.insert(key, value);
                        }
                    }
                }
            }
        } else {
            return Err(format!("Root element in '{}' is not a JSON object", path_str).into());
        }
    }

    let final_map: Map<String, Value> = accumulator.into_iter().collect();
    Ok(Value::Object(final_map))
}

pub enum ConflictStrategy {
    Overwrite,
    Skip,
    MergeObjects,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_json(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn test_basic_merge() {
        let file1 = create_temp_json(r#"{"a": 1, "b": 2}"#);
        let file2 = create_temp_json(r#"{"c": 3, "d": 4}"#);

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]).unwrap();

        let expected: Value = serde_json::from_str(r#"{"a": 1, "b": 2, "c": 3, "d": 4}"#).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_conflict_overwrite() {
        let file1 = create_temp_json(r#"{"key": "first"}"#);
        let file2 = create_temp_json(r#"{"key": "second"}"#);

        let result = merge_json_with_strategy(
            &[
                file1.path().to_str().unwrap(),
                file2.path().to_str().unwrap(),
            ],
            ConflictStrategy::Overwrite,
        ).unwrap();

        let expected: Value = serde_json::from_str(r#"{"key": "second"}"#).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_objects_strategy() {
        let file1 = create_temp_json(r#"{"config": {"port": 8080}}"#);
        let file2 = create_temp_json(r#"{"config": {"host": "localhost"}}"#);

        let result = merge_json_with_strategy(
            &[
                file1.path().to_str().unwrap(),
                file2.path().to_str().unwrap(),
            ],
            ConflictStrategy::MergeObjects,
        ).unwrap();

        let expected: Value = serde_json::from_str(
            r#"{"config": {"port": 8080, "host": "localhost"}}"#
        ).unwrap();
        assert_eq!(result, expected);
    }
}use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], deduplicate_by_key: Option<&str>) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_keys = HashSet::new();

    for path in paths {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: Value = serde_json::from_str(&contents)?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    if let Some(key) = deduplicate_by_key {
                        if let Some(obj) = item.as_object() {
                            if let Some(key_value) = obj.get(key) {
                                let key_str = key_value.to_string();
                                if !seen_keys.contains(&key_str) {
                                    seen_keys.insert(key_str);
                                    merged_array.push(item);
                                }
                                continue;
                            }
                        }
                    }
                    merged_array.push(item);
                }
            }
            _ => merged_array.push(json_value),
        }
    }

    Ok(json!(merged_array))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_arrays() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        let json1 = json!([{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]);
        let json2 = json!([{"id": 3, "name": "Charlie"}]);

        writeln!(file1, "{}", json1).unwrap();
        writeln!(file2, "{}", json2).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()], None).unwrap();
        assert_eq!(result.as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_deduplicate_by_key() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        let json1 = json!([{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]);
        let json2 = json!([{"id": 2, "name": "Robert"}, {"id": 3, "name": "Charlie"}]);

        writeln!(file1, "{}", json1).unwrap();
        writeln!(file2, "{}", json2).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()], Some("id")).unwrap();
        let array = result.as_array().unwrap();
        assert_eq!(array.len(), 3);
        let ids: Vec<i64> = array
            .iter()
            .filter_map(|v| v.get("id").and_then(|id| id.as_i64()))
            .collect();
        assert_eq!(ids, vec![1, 2, 3]);
    }
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
            if let (Value::Object(existing_obj), Value::Object(new_obj)) = (existing, &new_value) {
                let mut existing_map = existing_obj.clone();
                for (k, v) in new_obj {
                    merge_value(&mut existing_map, k.clone(), v.clone());
                }
                *existing = Value::Object(existing_map);
            } else if existing != &new_value {
                *existing = Value::Array(vec![existing.clone(), new_value]);
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
    fn test_merge_json() -> Result<(), Box<dyn std::error::Error>> {
        let file1 = NamedTempFile::new()?;
        let file2 = NamedTempFile::new()?;
        
        fs::write(&file1, r#"{"a": 1, "b": {"x": 10}}"#)?;
        fs::write(&file2, r#"{"a": 2, "b": {"y": 20}}"#)?;
        
        let result = merge_json_files(&[file1.path(), file2.path()])?;
        
        assert_eq!(result["a"], json!([1, 2]));
        assert_eq!(result["b"]["x"], json!(10));
        assert_eq!(result["b"]["y"], json!(20));
        
        Ok(())
    }
}