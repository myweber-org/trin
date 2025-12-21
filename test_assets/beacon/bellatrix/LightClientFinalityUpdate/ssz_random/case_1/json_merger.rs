use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs;

pub fn merge_json_files(file_paths: &[String], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_ids = HashSet::new();

    for file_path in file_paths {
        let content = fs::read_to_string(file_path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                        if !seen_ids.contains(id) {
                            seen_ids.insert(id.to_string());
                            merged_array.push(item);
                        }
                    } else {
                        merged_array.push(item);
                    }
                }
            }
            Value::Object(obj) => {
                if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                    if !seen_ids.contains(id) {
                        seen_ids.insert(id.to_string());
                        merged_array.push(json!(obj));
                    }
                } else {
                    merged_array.push(json!(obj));
                }
            }
            _ => return Err("Unsupported JSON structure".into()),
        }
    }

    let output_json = json!(merged_array);
    fs::write(output_path, output_json.to_string())?;
    Ok(())
}