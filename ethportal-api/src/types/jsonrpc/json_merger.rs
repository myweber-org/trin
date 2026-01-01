
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
    fn test_merge_json() -> Result<(), Box<dyn std::error::Error>> {
        let file1 = NamedTempFile::new()?;
        let file2 = NamedTempFile::new()?;

        fs::write(&file1, r#"{"a": 1, "b": {"x": 10}}"#)?;
        fs::write(&file2, r#"{"b": {"y": 20}, "c": 3}"#)?;

        let merged = merge_json_files(&[file1.path(), file2.path()])?;
        
        assert_eq!(merged["a"], 1);
        assert_eq!(merged["b"]["x"], 10);
        assert_eq!(merged["b"]["y"], 20);
        assert_eq!(merged["c"], 3);

        Ok(())
    }
}use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

type JsonValue = serde_json::Value;
type JsonObject = serde_json::Map<String, JsonValue>;

pub fn merge_json_files(file_paths: &[impl AsRef<Path>]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged = JsonObject::new();

    for path in file_paths {
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        let json_value: JsonValue = serde_json::from_str(&content)?;
        if let JsonValue::Object(obj) = json_value {
            merge_objects(&mut merged, obj);
        } else {
            return Err("Top-level JSON must be an object".into());
        }
    }

    Ok(JsonValue::Object(merged))
}

fn merge_objects(target: &mut JsonObject, source: JsonObject) {
    for (key, value) in source {
        match (target.get_mut(&key), value) {
            (Some(JsonValue::Object(existing_obj)), JsonValue::Object(new_obj)) => {
                merge_objects(existing_obj.as_object_mut().unwrap(), new_obj);
            }
            (Some(_), new_value) => {
                target.insert(key, new_value);
            }
            (None, new_value) => {
                target.insert(key, new_value);
            }
        }
    }
}

pub fn write_merged_json(output_path: impl AsRef<Path>, value: &JsonValue) -> Result<(), Box<dyn std::error::Error>> {
    let json_string = serde_json::to_string_pretty(value)?;
    fs::write(output_path, json_string)?;
    Ok(())
}