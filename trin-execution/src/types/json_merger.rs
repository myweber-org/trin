
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();
    
    for path in paths {
        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;
        
        if let Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    let existing = merged_map.get(&key).unwrap();
                    
                    if existing.is_object() && value.is_object() {
                        if let (Value::Object(existing_obj), Value::Object(new_obj)) = (existing, &value) {
                            let mut combined_obj = existing_obj.clone();
                            for (sub_key, sub_value) in new_obj {
                                combined_obj.insert(sub_key.clone(), sub_value.clone());
                            }
                            merged_map.insert(key, Value::Object(combined_obj));
                        }
                    } else if existing.is_array() && value.is_array() {
                        if let (Value::Array(existing_arr), Value::Array(new_arr)) = (existing, &value) {
                            let mut combined_arr = existing_arr.clone();
                            combined_arr.extend(new_arr.clone());
                            merged_map.insert(key, Value::Array(combined_arr));
                        }
                    } else {
                        merged_map.insert(key + "_conflict", value);
                    }
                } else {
                    merged_map.insert(key, value);
                }
            }
        }
    }
    
    let merged_json = Value::Object(merged_map);
    let pretty_json = serde_json::to_string_pretty(&merged_json)?;
    fs::write(output_path, pretty_json)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();
        
        fs::write(&file1, r#"{"name": "test", "values": [1, 2]}"#).unwrap();
        fs::write(&file2, r#"{"version": "1.0", "values": [3, 4]}"#).unwrap();
        
        let paths = [file1.path(), file2.path()];
        merge_json_files(&paths, output.path()).unwrap();
        
        let result = fs::read_to_string(output.path()).unwrap();
        assert!(result.contains("\"name\""));
        assert!(result.contains("\"version\""));
        assert!(result.contains("[1,2,3,4]"));
    }
}