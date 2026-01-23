use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for file_path in file_paths {
        let content = fs::read_to_string(file_path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;
        
        if let serde_json::Value::Array(arr) = json_value {
            merged_array.extend(arr);
        } else {
            merged_array.push(json_value);
        }
    }

    let output_json = serde_json::to_string_pretty(&merged_array)?;
    fs::write(output_path, output_json)?;
    
    Ok(())
}

pub fn merge_json_with_deduplication(
    file_paths: &[&str], 
    output_path: &str, 
    key_field: &str
) -> Result<(), Box<dyn std::error::Error>> {
    let mut unique_items = HashMap::new();

    for file_path in file_paths {
        let content = fs::read_to_string(file_path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;
        
        let items = match json_value {
            serde_json::Value::Array(arr) => arr,
            _ => vec![json_value],
        };

        for item in items {
            if let Some(obj) = item.as_object() {
                if let Some(key_value) = obj.get(key_field) {
                    if let Some(key_str) = key_value.as_str() {
                        unique_items.insert(key_str.to_string(), item.clone());
                    }
                }
            }
        }
    }

    let deduplicated_array: Vec<_> = unique_items.values().cloned().collect();
    let output_json = serde_json::to_string_pretty(&deduplicated_array)?;
    fs::write(output_path, output_json)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_merge() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), r#"[{"id": 1}, {"id": 2}]"#).unwrap();
        fs::write(file2.path(), r#"[{"id": 3}, {"id": 4}]"#).unwrap();

        merge_json_files(
            &[file1.path().to_str().unwrap(), file2.path().to_str().unwrap()],
            output_file.path().to_str().unwrap()
        ).unwrap();

        let content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 4);
    }
}use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for file_path in file_paths {
        let content = fs::read_to_string(file_path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object at the top level".into());
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
        let file1_content = r#"{"name": "test", "count": 42}"#;
        let file2_content = r#"{"enabled": true, "tags": ["rust", "json"]}"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), file1_content).unwrap();
        fs::write(file2.path(), file2_content).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths, output_file.path().to_str().unwrap());
        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();

        assert_eq!(parsed["name"], "test");
        assert_eq!(parsed["count"], 42);
        assert_eq!(parsed["enabled"], true);
        assert!(parsed["tags"].is_array());
    }
}