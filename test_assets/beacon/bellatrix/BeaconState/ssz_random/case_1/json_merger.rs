use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Vec<serde_json::Value>, String> {
    let mut merged_array = Vec::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str));
        }

        let file = File::open(path).map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents).map_err(|e| e.to_string())?;

        let json_value: serde_json::Value =
            serde_json::from_str(&contents).map_err(|e| e.to_string())?;

        match json_value {
            serde_json::Value::Array(arr) => {
                merged_array.extend(arr);
            }
            serde_json::Value::Object(obj) => {
                merged_array.push(serde_json::Value::Object(obj));
            }
            _ => {
                return Err(format!("Unsupported JSON structure in file: {}", path_str));
            }
        }
    }

    Ok(merged_array)
}

pub fn deduplicate_json_array(array: Vec<serde_json::Value>, key: &str) -> Vec<serde_json::Value> {
    let mut seen = HashMap::new();
    let mut deduped = Vec::new();

    for value in array {
        if let Some(obj) = value.as_object() {
            if let Some(id_value) = obj.get(key) {
                let id = id_value.to_string();
                if !seen.contains_key(&id) {
                    seen.insert(id.clone(), true);
                    deduped.push(value);
                }
            } else {
                deduped.push(value);
            }
        } else {
            deduped.push(value);
        }
    }

    deduped
}