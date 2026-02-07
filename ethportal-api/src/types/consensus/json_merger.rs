
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::Path;

use serde_json::{json, Value};

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), String> {
    let mut merged_array = Vec::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        if !path.exists() {
            return Err(format!("Input file not found: {}", input_path));
        }

        let file = File::open(path).map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents).map_err(|e| e.to_string())?;

        let json_value: Value = serde_json::from_str(&contents).map_err(|e| e.to_string())?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    merged_array.push(item);
                }
            }
            Value::Object(obj) => {
                merged_array.push(Value::Object(obj));
            }
            _ => {
                return Err(format!("Unsupported JSON structure in file: {}", input_path));
            }
        }
    }

    let output_value = Value::Array(merged_array);
    let output_json = serde_json::to_string_pretty(&output_value).map_err(|e| e.to_string())?;

    let mut output_file = File::create(output_path).map_err(|e| e.to_string())?;
    output_file
        .write_all(output_json.as_bytes())
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn merge_json_with_deduplication(
    input_paths: &[&str],
    output_path: &str,
    key_field: &str,
) -> Result<(), String> {
    let mut unique_map: HashMap<String, Value> = HashMap::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        if !path.exists() {
            return Err(format!("Input file not found: {}", input_path));
        }

        let contents = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let json_value: Value = serde_json::from_str(&contents).map_err(|e| e.to_string())?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    if let Value::Object(ref obj) = item {
                        if let Some(Value::String(key)) = obj.get(key_field) {
                            unique_map.insert(key.clone(), item.clone());
                        }
                    }
                }
            }
            Value::Object(obj) => {
                if let Some(Value::String(key)) = obj.get(key_field) {
                    unique_map.insert(key.clone(), Value::Object(obj));
                }
            }
            _ => {
                return Err(format!("Unsupported JSON structure in file: {}", input_path));
            }
        }
    }

    let merged_array: Vec<Value> = unique_map.into_values().collect();
    let output_value = Value::Array(merged_array);
    let output_json = serde_json::to_string_pretty(&output_value).map_err(|e| e.to_string())?;

    fs::write(output_path, output_json).map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1_content = r#"[{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]"#;
        let file2_content = r#"[{"id": 3, "name": "Charlie"}]"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), file1_content).unwrap();
        fs::write(file2.path(), file2_content).unwrap();

        let input_paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];
        let output_path = output_file.path().to_str().unwrap();

        let result = merge_json_files(&input_paths, output_path);
        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_path).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();
        assert!(parsed.is_array());
        let array = parsed.as_array().unwrap();
        assert_eq!(array.len(), 3);
    }

    #[test]
    fn test_merge_json_with_deduplication() {
        let file1_content = r#"[{"id": "a1", "name": "Alice"}, {"id": "a2", "name": "Bob"}]"#;
        let file2_content = r#"[{"id": "a1", "name": "Alice Updated"}, {"id": "a3", "name": "Charlie"}]"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), file1_content).unwrap();
        fs::write(file2.path(), file2_content).unwrap();

        let input_paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];
        let output_path = output_file.path().to_str().unwrap();

        let result = merge_json_with_deduplication(&input_paths, output_path, "id");
        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_path).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();
        let array = parsed.as_array().unwrap();
        assert_eq!(array.len(), 3);

        let mut ids: Vec<String> = array
            .iter()
            .filter_map(|v| v.get("id").and_then(|id| id.as_str()).map(String::from))
            .collect();
        ids.sort();
        assert_eq!(ids, vec!["a1", "a2", "a3"]);
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
            return Err(format!("Top-level JSON must be an object in {}", path.as_ref().display()));
        }
    }

    Ok(Value::Object(merged))
}

fn merge_objects(target: &mut Map<String, Value>, source: Map<String, Value>) {
    for (key, source_value) in source {
        match target.get_mut(&key) {
            Some(Value::Object(target_obj)) => {
                if let Value::Object(source_obj) = source_value {
                    merge_objects(target_obj, source_obj);
                } else {
                    target.insert(key, source_value);
                }
            }
            Some(Value::Array(target_arr)) => {
                if let Value::Array(source_arr) = source_value {
                    target_arr.extend(source_arr);
                } else {
                    target.insert(key, source_value);
                }
            }
            Some(_) => {
                target.insert(key, source_value);
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
    fn test_merge_basic_objects() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": 2}"#).unwrap();
        fs::write(&file2, r#"{"c": 3, "d": 4}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "a": 1,
            "b": 2,
            "c": 3,
            "d": 4
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_nested_objects() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"config": {"port": 8080}}"#).unwrap();
        fs::write(&file2, r#"{"config": {"host": "localhost"}}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "config": {
                "port": 8080,
                "host": "localhost"
            }
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_arrays() {
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