
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    let mut merged = Map::new();
    
    for path in paths {
        let content = fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        let json: Value = serde_json::from_str(&content).map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;
        
        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if merged.contains_key(&key) {
                    let existing = &merged[&key];
                    if existing != &value {
                        return Err(format!("Conflict detected for key '{}'", key));
                    }
                } else {
                    merged.insert(key, value);
                }
            }
        } else {
            return Err("Top-level JSON must be an object".to_string());
        }
    }
    
    Ok(Value::Object(merged))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        
        fs::write(&file1, r#"{"a": 1, "b": 2}"#).unwrap();
        fs::write(&file2, r#"{"c": 3, "d": 4}"#).unwrap();
        
        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let obj = result.as_object().unwrap();
        
        assert_eq!(obj.len(), 4);
        assert_eq!(obj["a"], 1);
        assert_eq!(obj["c"], 3);
    }
    
    #[test]
    fn test_merge_conflict() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        
        fs::write(&file1, r#"{"a": 1}"#).unwrap();
        fs::write(&file2, r#"{"a": 2}"#).unwrap();
        
        let result = merge_json_files(&[file1.path(), file2.path()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Conflict"));
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
            merge_objects(&mut merged, obj);
        }
    }

    Ok(Value::Object(merged))
}

fn merge_objects(target: &mut Map<String, Value>, source: Map<String, Value>) {
    for (key, source_value) in source {
        match target.get_mut(&key) {
            Some(target_value) => {
                if let (Value::Object(mut target_obj), Value::Object(source_obj)) = (target_value.clone(), source_value.clone()) {
                    let mut source_map = source_obj;
                    merge_objects(&mut target_obj, source_map);
                    target.insert(key, Value::Object(target_obj));
                } else if target_value != &source_value {
                    let merged_array = Value::Array(vec![target_value.clone(), source_value]);
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
    fn test_basic_merge() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": "test"}"#).unwrap();
        fs::write(&file2, r#"{"c": true, "d": null}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "a": 1,
            "b": "test",
            "c": true,
            "d": null
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_conflict_resolution() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "nested": {"x": 10}}"#).unwrap();
        fs::write(&file2, r#"{"a": 2, "nested": {"y": 20}}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        
        if let Value::Array(arr) = &result["a"] {
            assert!(arr.contains(&json!(1)));
            assert!(arr.contains(&json!(2)));
        } else {
            panic!("Expected array for key 'a'");
        }

        assert_eq!(result["nested"]["x"], 10);
        assert_eq!(result["nested"]["y"], 20);
    }
}