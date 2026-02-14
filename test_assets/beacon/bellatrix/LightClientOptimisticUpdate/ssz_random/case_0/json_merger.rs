
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), String> {
    if paths.is_empty() {
        return Err("No input files provided".to_string());
    }

    let mut merged_map = Map::new();
    let mut conflict_log = Vec::new();

    for (idx, path) in paths.iter().enumerate() {
        let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path.as_ref().display(), e))?;
        let reader = BufReader::new(file);
        let json: Map<String, Value> = serde_json::from_reader(reader)
            .map_err(|e| format!("Failed to parse {}: {}", path.as_ref().display(), e))?;

        for (key, value) in json {
            if let Some(existing) = merged_map.get(&key) {
                if existing != &value {
                    conflict_log.push(format!(
                        "Conflict at key '{}': file {} has {:?}, previous had {:?}",
                        key,
                        idx + 1,
                        value,
                        existing
                    ));
                    merged_map.insert(format!("{}_conflict_{}", key, idx + 1), value);
                }
            } else {
                merged_map.insert(key, value);
            }
        }
    }

    let mut output_map = Map::new();
    output_map.insert("data".to_string(), Value::Object(merged_map));
    
    if !conflict_log.is_empty() {
        output_map.insert("conflicts".to_string(), Value::Array(
            conflict_log.into_iter().map(Value::String).collect()
        ));
    }

    let output_file = File::create(&output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    serde_json::to_writer_pretty(output_file, &Value::Object(output_map))
        .map_err(|e| format!("Failed to write JSON: {}", e))?;

    Ok(())
}

pub fn find_unique_keys<P: AsRef<Path>>(paths: &[P]) -> Result<Vec<HashSet<String>>, String> {
    let mut key_sets = Vec::new();
    
    for path in paths {
        let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path.as_ref().display(), e))?;
        let reader = BufReader::new(file);
        let json: Map<String, Value> = serde_json::from_reader(reader)
            .map_err(|e| format!("Failed to parse {}: {}", path.as_ref().display(), e))?;
        
        key_sets.push(json.keys().cloned().collect());
    }
    
    Ok(key_sets)
}use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_ids = HashMap::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", file_path);
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    if let Some(obj) = item.as_object() {
                        if let Some(id_value) = obj.get("id") {
                            if let Some(id_str) = id_value.as_str() {
                                if seen_ids.contains_key(id_str) {
                                    eprintln!("Duplicate ID '{}' found, skipping.", id_str);
                                    continue;
                                }
                                seen_ids.insert(id_str.to_string(), true);
                            }
                        }
                    }
                    merged_array.push(item);
                }
            }
            Value::Object(_) => {
                if let Some(id_value) = json_value.get("id") {
                    if let Some(id_str) = id_value.as_str() {
                        if seen_ids.contains_key(id_str) {
                            eprintln!("Duplicate ID '{}' found, skipping.", id_str);
                            continue;
                        }
                        seen_ids.insert(id_str.to_string(), true);
                    }
                }
                merged_array.push(json_value);
            }
            _ => eprintln!("Warning: JSON root is not an array or object in file {}, skipping.", file_path),
        }
    }

    let output_json = json!(merged_array);
    fs::write(output_path, serde_json::to_string_pretty(&output_json)?)?;
    println!("Successfully merged {} items into {}", merged_array.len(), output_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1_content = r#"[{"id": "1", "name": "Alice"}, {"id": "2", "name": "Bob"}]"#;
        let file2_content = r#"[{"id": "3", "name": "Charlie"}, {"id": "1", "name": "Duplicate"}]"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), file1_content).unwrap();
        fs::write(file2.path(), file2_content).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths, output_file.path().to_str().unwrap());
        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    let mut merged = Map::new();
    
    for path in paths {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        
        let json: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;
        
        if let Value::Object(obj) = json {
            merge_objects(&mut merged, obj);
        } else {
            return Err("Top-level JSON must be an object".to_string());
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
                        source_value
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
    fn test_merge_basic() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        
        fs::write(&file1, r#"{"a": 1, "b": 2}"#).unwrap();
        fs::write(&file2, r#"{"b": 3, "c": 4}"#).unwrap();
        
        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "a": 1,
            "b": [2, 3],
            "c": 4
        });
        
        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_nested() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        
        fs::write(&file1, r#"{"config": {"timeout": 30}}"#).unwrap();
        fs::write(&file2, r#"{"config": {"retries": 5}}"#).unwrap();
        
        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "config": {
                "timeout": 30,
                "retries": 5
            }
        });
        
        assert_eq!(result, expected);
    }
}