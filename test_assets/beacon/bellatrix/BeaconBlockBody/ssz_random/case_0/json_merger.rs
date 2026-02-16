
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufReader, Write};
use std::path::Path;

use serde_json::{Value, json};

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> io::Result<()> {
    let mut merged_array = Vec::new();

    for path in paths {
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
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("JSON file must contain an array or object: {:?}", path.as_ref()),
                ));
            }
        }
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &json!(merged_array))?;

    Ok(())
}

pub fn merge_json_with_deduplication<P: AsRef<Path>>(
    paths: &[P],
    output_path: P,
    key_field: &str,
) -> io::Result<()> {
    let mut unique_map: HashMap<String, Value> = HashMap::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        let items = match json_value {
            Value::Array(arr) => arr,
            Value::Object(obj) => vec![Value::Object(obj)],
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid JSON structure in file: {:?}", path.as_ref()),
                ));
            }
        };

        for item in items {
            if let Value::Object(map) = item {
                if let Some(key_value) = map.get(key_field) {
                    let key = key_value.to_string();
                    unique_map.insert(key, Value::Object(map));
                }
            }
        }
    }

    let deduplicated_array: Vec<Value> = unique_map.into_values().collect();
    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &json!(deduplicated_array))?;

    Ok(())
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged: Map<String, Value> = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if merged.contains_key(&key) {
                    let existing = &merged[&key];
                    if existing != &value {
                        let resolved = resolve_conflict(&key, existing, &value);
                        merged.insert(key, resolved);
                    }
                } else {
                    merged.insert(key, value);
                }
            }
        }
    }

    let output_json = Value::Object(merged);
    let output_str = serde_json::to_string_pretty(&output_json)?;
    fs::write(output_path, output_str)?;
    Ok(())
}

fn resolve_conflict(key: &str, v1: &Value, v2: &Value) -> Value {
    match (v1, v2) {
        (Value::Array(a1), Value::Array(a2)) => {
            let mut combined = a1.clone();
            combined.extend(a2.clone());
            Value::Array(combined)
        }
        (Value::Number(n1), Value::Number(n2)) => {
            if n1.as_f64().unwrap_or(0.0) > n2.as_f64().unwrap_or(0.0) {
                v1.clone()
            } else {
                v2.clone()
            }
        }
        _ => v2.clone()
    }
}