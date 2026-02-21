use serde_json::{json, Value};
use std::fs;
use std::io;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, io::Error> {
    let mut merged_array = Vec::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let parsed: Value = serde_json::from_str(&content)?;
        merged_array.push(parsed);
    }

    Ok(json!(merged_array))
}

pub fn write_merged_json<P: AsRef<Path>>(output_path: P, value: &Value) -> Result<(), io::Error> {
    let json_string = serde_json::to_string_pretty(value)?;
    fs::write(output_path, json_string)
}