use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[impl AsRef<Path>]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for path in file_paths {
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let parsed: JsonValue = serde_json::from_str(&contents)?;
        
        match parsed {
            JsonValue::Array(arr) => {
                merged_array.extend(arr);
            }
            JsonValue::Object(obj) => {
                merged_array.push(JsonValue::Object(obj));
            }
            _ => {
                return Err("Each JSON file must contain either an array or an object".into());
            }
        }
    }

    Ok(JsonValue::Array(merged_array))
}

pub fn deduplicate_json_array(array: JsonValue, key: &str) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let arr = array.as_array().ok_or("Input must be a JSON array")?;
    
    let mut seen = HashMap::new();
    let mut deduplicated = Vec::new();

    for item in arr {
        if let Some(obj) = item.as_object() {
            if let Some(value) = obj.get(key) {
                let key_string = value.to_string();
                if !seen.contains_key(&key_string) {
                    seen.insert(key_string.clone(), true);
                    deduplicated.push(item.clone());
                }
            }
        }
    }

    Ok(JsonValue::Array(deduplicated))
}

pub fn write_merged_json(output_path: impl AsRef<Path>, data: &JsonValue) -> Result<(), Box<dyn std::error::Error>> {
    let formatted = serde_json::to_string_pretty(data)?;
    fs::write(output_path, formatted)?;
    Ok(())
}