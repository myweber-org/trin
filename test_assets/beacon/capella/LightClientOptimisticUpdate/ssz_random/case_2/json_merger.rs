
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(input_paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path in input_paths {
        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    let merged_json = Value::Object(merged_map);
    let json_string = serde_json::to_string_pretty(&merged_json)?;
    fs::write(output_path, json_string)?;

    Ok(())
}