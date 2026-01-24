
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    if paths.is_empty() {
        return Err("No input files provided".to_string());
    }

    let mut merged_map = Map::new();

    for path in paths {
        let content = fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        let json_value: Value = serde_json::from_str(&content).map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                if merged_map.contains_key(&key) {
                    let existing = merged_map.get(&key).unwrap();
                    if existing != &value {
                        return Err(format!("Conflict detected for key '{}'", key));
                    }
                } else {
                    merged_map.insert(key, value);
                }
            }
        } else {
            return Err("Top-level JSON must be an object".to_string());
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn write_merged_json<P: AsRef<Path>>(output_path: P, value: &Value) -> Result<(), String> {
    let json_string = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    fs::write(output_path, json_string).map_err(|e| e.to_string())?;
    Ok(())
}