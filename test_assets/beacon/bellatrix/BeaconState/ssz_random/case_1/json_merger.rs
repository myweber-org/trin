
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
    
    let output_value = Value::Object(merged);
    let output_json = serde_json::to_string_pretty(&output_value)?;
    fs::write(output_path, output_json)?;
    
    Ok(())
}

fn merge_objects(base: &mut Map<String, Value>, new: Map<String, Value>) {
    for (key, new_value) in new {
        match (base.get(&key), new_value) {
            (Some(Value::Object(existing_obj)), Value::Object(new_obj)) => {
                let mut existing_map = existing_obj.clone()
                    .as_object()
                    .unwrap()
                    .clone();
                merge_objects(&mut existing_map, new_obj);
                base.insert(key, Value::Object(existing_map));
            }
            (Some(Value::Array(existing_arr)), Value::Array(new_arr)) => {
                let mut combined = existing_arr.clone();
                combined.extend(new_arr);
                base.insert(key, Value::Array(combined));
            }
            (Some(existing), new_val) if existing != &new_val => {
                let conflict_key = format!("{}_conflict", key);
                base.insert(conflict_key, new_val);
            }
            (_, new_val) => {
                base.insert(key, new_val);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use serde_json::json;

    #[test]
    fn test_basic_merge() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();
        
        fs::write(&file1, r#"{"name": "test", "value": 1}"#).unwrap();
        fs::write(&file2, r#"{"description": "sample", "value": 2}"#).unwrap();
        
        merge_json_files(&[&file1, &file2], &output).unwrap();
        
        let result_content = fs::read_to_string(output).unwrap();
        let result: Value = serde_json::from_str(&result_content).unwrap();
        
        assert_eq!(result["name"], "test");
        assert_eq!(result["description"], "sample");
        assert_eq!(result["value"], 2);
    }
}