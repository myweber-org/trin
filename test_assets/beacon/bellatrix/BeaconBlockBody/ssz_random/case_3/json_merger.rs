
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
    resolution: ConflictResolution,
) -> Result<Map<String, Value>, String> {
    let mut result = first.clone();
    let mut conflicts = Vec::new();

    for (key, value2) in second {
        match result.get(key) {
            Some(value1) => {
                if value1 != value2 {
                    match resolution {
                        ConflictResolution::PreferFirst => continue,
                        ConflictResolution::PreferSecond => {
                            result.insert(key.clone(), value2.clone());
                        }
                        ConflictResolution::MergeArrays => {
                            if let (Value::Array(arr1), Value::Array(arr2)) = (value1, value2) {
                                let mut merged = arr1.clone();
                                merged.extend(arr2.clone());
                                result.insert(key.clone(), Value::Array(merged));
                            } else {
                                conflicts.push(key.clone());
                            }
                        }
                        ConflictResolution::FailOnConflict => {
                            return Err(format!("Conflict detected on key: {}", key));
                        }
                    }
                }
            }
            None => {
                result.insert(key.clone(), value2.clone());
            }
        }
    }

    if !conflicts.is_empty() && matches!(resolution, ConflictResolution::MergeArrays) {
        return Err(format!(
            "Cannot merge non-array values for keys: {:?}",
            conflicts
        ));
    }

    Ok(result)
}

pub fn find_common_keys(first: &Map<String, Value>, second: &Map<String, Value>) -> HashSet<String> {
    first.keys().filter(|k| second.contains_key(*k)).cloned().collect()
}
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::Path;

use serde_json::{Value, json};

pub fn merge_json_files(input_paths: &[String], output_path: &str) -> Result<(), String> {
    let mut merged_array = Vec::new();

    for path_str in input_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str));
        }

        let mut file = File::open(path).map_err(|e| e.to_string())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|e| e.to_string())?;

        let json_value: Value = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse JSON from {}: {}", path_str, e))?;

        match json_value {
            Value::Array(arr) => {
                merged_array.extend(arr);
            }
            Value::Object(_) => {
                merged_array.push(json_value);
            }
            _ => {
                return Err(format!("Unsupported JSON structure in {}", path_str));
            }
        }
    }

    let output_file = File::create(output_path).map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(output_file, &json!(merged_array))
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn merge_json_with_key_deduplication(
    input_paths: &[String],
    output_path: &str,
    unique_key: &str,
) -> Result<(), String> {
    let mut seen_keys = HashMap::new();
    let mut deduplicated_array = Vec::new();

    for path_str in input_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str));
        }

        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)
            .map_err(|e| format!("Failed to parse JSON from {}: {}", path_str, e))?;

        if let Value::Array(arr) = json_value {
            for item in arr {
                if let Some(obj) = item.as_object() {
                    if let Some(key_value) = obj.get(unique_key) {
                        let key_str = key_value.to_string();
                        if !seen_keys.contains_key(&key_str) {
                            seen_keys.insert(key_str.clone(), true);
                            deduplicated_array.push(item);
                        }
                    } else {
                        return Err(format!("Missing key '{}' in object from {}", unique_key, path_str));
                    }
                }
            }
        } else {
            return Err(format!("Expected JSON array in {}", path_str));
        }
    }

    let output_file = File::create(output_path).map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(output_file, &json!(deduplicated_array))
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_merge() {
        let file1_content = r#"[{"id": 1, "name": "Alice"}]"#;
        let file2_content = r#"[{"id": 2, "name": "Bob"}]"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), file1_content).unwrap();
        fs::write(file2.path(), file2_content).unwrap();

        let input_paths = vec![
            file1.path().to_str().unwrap().to_string(),
            file2.path().to_str().unwrap().to_string(),
        ];

        let result = merge_json_files(&input_paths, output_file.path().to_str().unwrap());
        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 2);
    }
}