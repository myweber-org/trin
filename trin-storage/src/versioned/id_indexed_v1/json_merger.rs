use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[impl AsRef<Path>]) -> Result<JsonValue, String> {
    let mut merged_array = Vec::new();

    for path in file_paths {
        let file = File::open(path.as_ref())
            .map_err(|e| format!("Failed to open {}: {}", path.as_ref().display(), e))?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content)
            .map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;

        let json_value: JsonValue = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        match json_value {
            JsonValue::Array(arr) => merged_array.extend(arr),
            JsonValue::Object(obj) => merged_array.push(JsonValue::Object(obj)),
            _ => return Err(format!("JSON root must be array or object in {}", path.as_ref().display()))
        }
    }

    Ok(JsonValue::Array(merged_array))
}

pub fn deduplicate_json_array(array: JsonValue, key_field: &str) -> Result<JsonValue, String> {
    let arr = array.as_array()
        .ok_or_else(|| "Input must be a JSON array".to_string())?;

    let mut seen = HashMap::new();
    let mut deduplicated = Vec::new();

    for item in arr {
        if let Some(obj) = item.as_object() {
            if let Some(key_value) = obj.get(key_field) {
                let key_string = key_value.to_string();
                if !seen.contains_key(&key_string) {
                    seen.insert(key_string.clone(), true);
                    deduplicated.push(item.clone());
                }
            }
        }
    }

    Ok(JsonValue::Array(deduplicated))
}

pub fn write_merged_json(output_path: impl AsRef<Path>, json_value: &JsonValue) -> Result<(), String> {
    let pretty_json = serde_json::to_string_pretty(json_value)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

    fs::write(output_path.as_ref(), pretty_json)
        .map_err(|e| format!("Failed to write to {}: {}", output_path.as_ref().display(), e))
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
                for (nested_key, nested_value) in new_obj {
                    merge_value(existing_obj, nested_key.clone(), nested_value.clone());
                }
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
        fs::write(&file2, r#"{"b": {"y": 20}, "c": 3}"#)?;

        let result = merge_json_files(&[file1.path(), file2.path()])?;
        let expected = json!({
            "a": 1,
            "b": {"x": 10, "y": 20},
            "c": 3
        });

        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_conflict_resolution() -> Result<(), Box<dyn std::error::Error>> {
        let file1 = NamedTempFile::new()?;
        let file2 = NamedTempFile::new()?;

        fs::write(&file1, r#"{"a": 1}"#)?;
        fs::write(&file2, r#"{"a": 2}"#)?;

        let result = merge_json_files(&[file1.path(), file2.path()])?;
        let expected = json!({
            "a": [1, 2]
        });

        assert_eq!(result, expected);
        Ok(())
    }
}