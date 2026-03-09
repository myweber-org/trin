use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<HashMap<String, serde_json::Value>, Box<dyn std::error::Error>> {
    let mut merged = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                merged.insert(key, value);
            }
        }
    }

    Ok(merged)
}

pub fn write_merged_json(output_path: &str, data: &HashMap<String, serde_json::Value>) -> Result<(), Box<dyn std::error::Error>> {
    let json_string = serde_json::to_string_pretty(data)?;
    fs::write(output_path, json_string)?;
    Ok(())
}use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for path_str in input_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("Input file not found: {}", path_str).into());
        }

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: serde_json::Value = serde_json::from_str(&contents)?;
        merged_array.push(json_value);
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &merged_array)?;

    Ok(())
}

pub fn merge_json_with_deduplication(
    input_paths: &[&str],
    output_path: &str,
    key_field: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut unique_map: HashMap<String, serde_json::Value> = HashMap::new();

    for path_str in input_paths {
        let path = Path::new(path_str);
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_array: Vec<serde_json::Value> = serde_json::from_str(&contents)?;

        for item in json_array {
            if let Some(obj) = item.as_object() {
                if let Some(key_value) = obj.get(key_field) {
                    if let Some(key_str) = key_value.as_str() {
                        unique_map.insert(key_str.to_string(), item);
                    }
                }
            }
        }
    }

    let deduplicated: Vec<_> = unique_map.into_values().collect();
    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &deduplicated)?;

    Ok(())
}