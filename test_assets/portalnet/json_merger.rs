
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::Path;

use serde_json::{Value, json};

pub fn merge_json_files(input_paths: &[String], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for path_str in input_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            eprintln!("Warning: File {} does not exist, skipping.", path_str);
            continue;
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        match json_value {
            Value::Array(arr) => {
                merged_array.extend(arr);
            }
            Value::Object(obj) => {
                merged_array.push(Value::Object(obj));
            }
            _ => {
                eprintln!("Warning: File {} does not contain a JSON object or array, skipping.", path_str);
            }
        }
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &json!(merged_array))?;

    Ok(())
}

pub fn merge_json_with_key(input_paths: &[String], output_path: &str, key_field: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map: HashMap<String, Value> = HashMap::new();

    for path_str in input_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            eprintln!("Warning: File {} does not exist, skipping.", path_str);
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    if let Some(obj) = item.as_object() {
                        if let Some(key_value) = obj.get(key_field) {
                            if let Some(key_str) = key_value.as_str() {
                                merged_map.insert(key_str.to_string(), Value::Object(obj.clone()));
                            }
                        }
                    }
                }
            }
            Value::Object(obj) => {
                if let Some(key_value) = obj.get(key_field) {
                    if let Some(key_str) = key_value.as_str() {
                        merged_map.insert(key_str.to_string(), Value::Object(obj.clone()));
                    }
                }
            }
            _ => {
                eprintln!("Warning: File {} does not contain a JSON object or array, skipping.", path_str);
            }
        }
    }

    let output_file = File::create(output_path)?;
    let merged_vec: Vec<&Value> = merged_map.values().collect();
    serde_json::to_writer_pretty(output_file, &json!(merged_vec))?;

    Ok(())
}