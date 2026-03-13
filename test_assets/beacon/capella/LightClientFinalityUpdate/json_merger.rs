
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

fn merge_objects(base: &mut Map<String, Value>, new: Map<String, Value>) {
    for (key, value) in new {
        if let Some(existing) = base.get_mut(&key) {
            if existing.is_object() && value.is_object() {
                if let (Value::Object(ref mut base_obj), Value::Object(new_obj)) = (existing, value) {
                    merge_objects(base_obj, new_obj);
                }
            } else {
                *existing = value;
            }
        } else {
            base.insert(key, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use serde_json::json;

    #[test]
    fn test_merge_json() -> Result<(), Box<dyn std::error::Error>> {
        let file1 = NamedTempFile::new()?;
        let file2 = NamedTempFile::new()?;
        let output = NamedTempFile::new()?;
        
        fs::write(&file1, r#"{"a": 1, "b": {"x": 10}}"#)?;
        fs::write(&file2, r#"{"b": {"y": 20}, "c": 3}"#)?;
        
        merge_json_files(&[file1.path(), file2.path()], output.path())?;
        
        let result_content = fs::read_to_string(output.path())?;
        let result: Value = serde_json::from_str(&result_content)?;
        
        let expected = json!({
            "a": 1,
            "b": {"x": 10, "y": 20},
            "c": 3
        });
        
        assert_eq!(result, expected);
        Ok(())
    }
}