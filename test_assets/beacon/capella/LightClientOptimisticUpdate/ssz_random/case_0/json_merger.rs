
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"name": "Alice", "age": 30}"#).unwrap();
        writeln!(file2, r#"{"city": "London", "active": true}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]).unwrap();

        assert_eq!(result["name"], "Alice");
        assert_eq!(result["age"], 30);
        assert_eq!(result["city"], "London");
        assert_eq!(result["active"], true);
    }

    #[test]
    fn test_merge_with_missing_file() {
        let mut file1 = NamedTempFile::new().unwrap();
        writeln!(file1, r#"{"data": "test"}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            "non_existent_file.json",
        ]).unwrap();

        assert_eq!(result["data"], "test");
    }
}
use std::collections::HashMap;
use std::fs;
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[impl AsRef<Path>]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged = HashMap::new();

    for path in file_paths {
        let content = fs::read_to_string(path)?;
        let json_data: JsonValue = serde_json::from_str(&content)?;

        if let JsonValue::Object(obj) = json_data {
            for (key, value) in obj {
                merged.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    Ok(JsonValue::Object(merged.into_iter().collect()))
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
        writeln!(file2, r#"{"city": "London", "active": true}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected: JsonValue = serde_json::from_str(
            r#"{"name": "Alice", "age": 30, "city": "London", "active": true}"#
        ).unwrap();

        assert_eq!(result, expected);
    }
}
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

pub struct JsonMerger {
    conflict_resolution: ConflictResolution,
}

pub enum ConflictResolution {
    PreferFirst,
    PreferSecond,
    MergeObjects,
    UseCustom(Box<dyn Fn(&Value, &Value) -> Value>),
}

impl JsonMerger {
    pub fn new(resolution: ConflictResolution) -> Self {
        JsonMerger {
            conflict_resolution: resolution,
        }
    }

    pub fn merge_files<P: AsRef<Path>>(&self, paths: &[P]) -> Result<Value, String> {
        if paths.is_empty() {
            return Err("No files provided".to_string());
        }

        let mut merged = Value::Object(Map::new());

        for path in paths {
            let file = File::open(path).map_err(|e| e.to_string())?;
            let reader = BufReader::new(file);
            let json: Value = serde_json::from_reader(reader).map_err(|e| e.to_string())?;

            merged = self.merge_values(merged, json);
        }

        Ok(merged)
    }

    fn merge_values(&self, mut base: Value, new: Value) -> Value {
        match (base, new) {
            (Value::Object(mut base_map), Value::Object(new_map)) => {
                for (key, new_val) in new_map {
                    if let Some(existing_val) = base_map.remove(&key) {
                        let merged = self.merge_values(existing_val, new_val);
                        base_map.insert(key, merged);
                    } else {
                        base_map.insert(key, new_val);
                    }
                }
                Value::Object(base_map)
            }
            (Value::Array(mut base_arr), Value::Array(new_arr)) => {
                base_arr.extend(new_arr);
                Value::Array(base_arr)
            }
            (base_val, new_val) => {
                if base_val == new_val {
                    base_val
                } else {
                    match &self.conflict_resolution {
                        ConflictResolution::PreferFirst => base_val,
                        ConflictResolution::PreferSecond => new_val,
                        ConflictResolution::MergeObjects => {
                            let mut map = Map::new();
                            map.insert("first".to_string(), base_val);
                            map.insert("second".to_string(), new_val);
                            Value::Object(map)
                        }
                        ConflictResolution::UseCustom(func) => func(&base_val, &new_val),
                    }
                }
            }
        }
    }

    pub fn write_to_file<P: AsRef<Path>>(&self, value: &Value, path: P) -> Result<(), String> {
        let json_string = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
        let mut file = File::create(path).map_err(|e| e.to_string())?;
        file.write_all(json_string.as_bytes()).map_err(|e| e.to_string())?;
        Ok(())
    }
}

pub fn create_priority_merger(priority_map: HashMap<String, usize>) -> JsonMerger {
    let resolver = move |a: &Value, b: &Value| -> Value {
        let a_str = a.to_string();
        let b_str = b.to_string();
        
        let a_priority = priority_map.get(&a_str).unwrap_or(&0);
        let b_priority = priority_map.get(&b_str).unwrap_or(&0);
        
        if a_priority >= b_priority {
            a.clone()
        } else {
            b.clone()
        }
    };

    JsonMerger::new(ConflictResolution::UseCustom(Box::new(resolver)))
}