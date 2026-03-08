
use serde_json::{Map, Value};
use std::collections::HashMap;
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

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                merged_map.insert(key, value);
            }
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
            continue;
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
                            if let (Value::Object(existing_obj), Value::Object(new_obj)) =
                                (existing, &value)
                            {
                                let mut merged = existing_obj.clone();
                                for (k, v) in new_obj {
                                    merged.insert(k.clone(), v.clone());
                                }
                                accumulator.insert(key, Value::Object(merged));
                            } else {
                                accumulator.insert(key, value);
                            }
                        } else {
                            accumulator.insert(key, value);
                        }
                    }
                }
            }
        }
    }

    let final_map: Map<String, Value> = accumulator.into_iter().collect();
    Ok(Value::Object(final_map))
}

#[derive(Clone, Copy)]
pub enum ConflictStrategy {
    Overwrite,
    Skip,
    MergeObjects,
}use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[impl AsRef<Path>]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for path in file_paths {
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        let json_data: JsonValue = serde_json::from_str(&content)?;
        
        if let JsonValue::Array(arr) = json_data {
            merged_array.extend(arr);
        } else {
            merged_array.push(json_data);
        }
    }

    Ok(JsonValue::Array(merged_array))
}

pub fn merge_with_key_deduplication(
    file_paths: &[impl AsRef<Path>],
    key_field: &str,
) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut unique_map: HashMap<String, JsonValue> = HashMap::new();

    for path in file_paths {
        let content = fs::read_to_string(path.as_ref())?;
        let json_data: JsonValue = serde_json::from_str(&content)?;

        let items = match json_data {
            JsonValue::Array(arr) => arr,
            _ => vec![json_data],
        };

        for item in items {
            if let Some(obj) = item.as_object() {
                if let Some(key_value) = obj.get(key_field) {
                    if let Some(key_str) = key_value.as_str() {
                        unique_map.insert(key_str.to_string(), item);
                    }
                }
            }
        }
    }

    let deduplicated: Vec<JsonValue> = unique_map.into_values().collect();
    Ok(JsonValue::Array(deduplicated))
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
    fn test_merge_basic() {
        let file1 = create_temp_json(r#"[{"id": 1}, {"id": 2}]"#);
        let file2 = create_temp_json(r#"[{"id": 3}, {"id": 4}]"#);
        
        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        assert_eq!(result.as_array().unwrap().len(), 4);
    }

    #[test]
    fn test_deduplication() {
        let file1 = create_temp_json(r#"[{"id": "a", "value": 1}]"#);
        let file2 = create_temp_json(r#"[{"id": "a", "value": 2}]"#);
        
        let result = merge_with_key_deduplication(&[file1.path(), file2.path()], "id").unwrap();
        assert_eq!(result.as_array().unwrap().len(), 1);
    }
}