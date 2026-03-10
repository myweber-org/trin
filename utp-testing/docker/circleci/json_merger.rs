
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        if !path.exists() {
            eprintln!("Warning: File '{}' not found, skipping.", input_path);
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            eprintln!("Warning: '{}' does not contain a JSON object, skipping.", input_path);
        }
    }

    let merged_json = Value::Object(merged_map);
    let pretty_json = serde_json::to_string_pretty(&merged_json)?;
    fs::write(output_path, pretty_json)?;

    println!("Successfully merged {} file(s) into '{}'.", input_paths.len(), output_path);
    Ok(())
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
        let output_file = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"name": "Alice", "age": 30}"#).unwrap();
        writeln!(file2, r#"{"city": "Berlin", "country": "DE"}"#).unwrap();

        let inputs = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];
        let output_path = output_file.path().to_str().unwrap();

        let result = merge_json_files(&inputs, output_path);
        assert!(result.is_ok());

        let content = fs::read_to_string(output_path).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["name"], "Alice");
        assert_eq!(parsed["age"], 30);
        assert_eq!(parsed["city"], "Berlin");
        assert_eq!(parsed["country"], "DE");
    }
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    let mut merged = Map::new();
    
    for path in paths {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        
        let json: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;
        
        if let Value::Object(obj) = json {
            merge_objects(&mut merged, obj);
        } else {
            return Err("Top-level JSON must be an object".to_string());
        }
    }
    
    Ok(Value::Object(merged))
}

fn merge_objects(base: &mut Map<String, Value>, new: Map<String, Value>) {
    for (key, new_value) in new {
        match base.get_mut(&key) {
            Some(existing_value) => {
                if let (Value::Object(mut existing_obj), Value::Object(new_obj)) = (existing_value, new_value) {
                    merge_objects(&mut existing_obj, new_obj);
                } else {
                    *existing_value = new_value;
                }
            }
            None => {
                base.insert(key, new_value);
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
    fn test_merge_basic() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        
        fs::write(&file1, r#"{"a": 1, "b": {"x": 10}}"#).unwrap();
        fs::write(&file2, r#"{"c": 3, "b": {"y": 20}}"#).unwrap();
        
        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "a": 1,
            "b": {"x": 10, "y": 20},
            "c": 3
        });
        
        assert_eq!(result, expected);
    }
}