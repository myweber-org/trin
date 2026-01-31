
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde_json::{Value, Map};

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str).into());
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn merge_json_strings(json_strings: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for json_str in json_strings {
        let json_value: Value = serde_json::from_str(json_str)?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON string must represent a JSON object".into());
        }
    }

    Ok(Value::Object(merged_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_json_strings() {
        let json1 = r#"{"name": "Alice", "age": 30}"#;
        let json2 = r#"{"city": "New York", "country": "USA"}"#;
        
        let result = merge_json_strings(&[json1, json2]).unwrap();
        let expected = json!({
            "name": "Alice",
            "age": 30,
            "city": "New York",
            "country": "USA"
        });
        
        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_with_duplicate_keys() {
        let json1 = r#"{"id": 1, "value": "first"}"#;
        let json2 = r#"{"id": 2, "status": "active"}"#;
        
        let result = merge_json_strings(&[json1, json2]).unwrap();
        let expected = json!({
            "id": 2,
            "value": "first",
            "status": "active"
        });
        
        assert_eq!(result, expected);
    }
}