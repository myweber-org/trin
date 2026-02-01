
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged = Map::new();
    
    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;
        
        if let Value::Object(obj) = json {
            merge_objects(&mut merged, obj);
        }
    }
    
    let output_json = Value::Object(merged);
    let serialized = serde_json::to_string_pretty(&output_json)?;
    fs::write(output_path, serialized)?;
    
    Ok(())
}

fn merge_objects(base: &mut Map<String, Value>, extension: Map<String, Value>) {
    for (key, value) in extension {
        if !base.contains_key(&key) {
            base.insert(key.clone(), value);
            continue;
        }
        
        let existing = base.get_mut(&key).unwrap();
        match (existing, value) {
            (Value::Object(ref mut base_obj), Value::Object(ext_obj)) => {
                merge_objects(base_obj, ext_obj);
            }
            (Value::Array(ref mut base_arr), Value::Array(ext_arr)) => {
                base_arr.extend(ext_arr);
            }
            _ => {
                *existing = value;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_merge_basic() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();
        
        fs::write(&file1, r#"{"a": 1, "b": {"x": 10}}"#).unwrap();
        fs::write(&file2, r#"{"c": 3, "b": {"y": 20}}"#).unwrap();
        
        merge_json_files(&[file1.path(), file2.path()], output.path()).unwrap();
        
        let result = fs::read_to_string(output.path()).unwrap();
        assert!(result.contains("\"a\": 1"));
        assert!(result.contains("\"c\": 3"));
        assert!(result.contains("\"x\": 10"));
        assert!(result.contains("\"y\": 20"));
    }
}