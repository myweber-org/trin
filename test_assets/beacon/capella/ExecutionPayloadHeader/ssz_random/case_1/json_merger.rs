
use serde_json::{Map, Value};
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file1.json> [file2.json ...]", args[0]);
        process::exit(1);
    }

    let mut merged = Map::new();

    for file_path in &args[1..] {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading {}: {}", file_path, e);
                process::exit(1);
            }
        };

        let json_data: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Error parsing JSON from {}: {}", file_path, e);
                process::exit(1);
            }
        };

        if let Value::Object(map) = json_data {
            for (key, value) in map {
                if merged.contains_key(&key) {
                    eprintln!("Warning: Key '{}' from {} overwrites previous value", key, file_path);
                }
                merged.insert(key, value);
            }
        } else {
            eprintln!("Error: {} does not contain a JSON object at root", file_path);
            process::exit(1);
        }
    }

    let output = Value::Object(merged);
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, String> {
    let mut merged_array = Vec::new();
    let mut seen_ids = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str));
        }

        let file = File::open(path).map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents).map_err(|e| e.to_string())?;

        let json_value: Value = serde_json::from_str(&contents).map_err(|e| e.to_string())?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    if let Some(obj) = item.as_object() {
                        if let Some(id_value) = obj.get("id") {
                            let id_str = id_value.to_string();
                            if !seen_ids.contains_key(&id_str) {
                                seen_ids.insert(id_str.clone(), true);
                                merged_array.push(item);
                            }
                        } else {
                            merged_array.push(item);
                        }
                    } else {
                        merged_array.push(item);
                    }
                }
            }
            Value::Object(_) => merged_array.push(json_value),
            _ => return Err("JSON root must be an array or object".to_string()),
        }
    }

    Ok(json!(merged_array))
}

pub fn write_merged_json(output_path: &str, value: &Value) -> Result<(), String> {
    let file = File::create(output_path).map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(file, value).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_arrays() {
        let file1_content = r#"[{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]"#;
        let file2_content = r#"[{"id": 2, "name": "Bob"}, {"id": 3, "name": "Charlie"}]"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(file1.path(), file1_content).unwrap();
        fs::write(file2.path(), file2_content).unwrap();

        let paths = &[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(paths).unwrap();
        let array = result.as_array().unwrap();

        assert_eq!(array.len(), 3);
        assert!(array.iter().any(|v| v["id"] == 1));
        assert!(array.iter().any(|v| v["id"] == 2));
        assert!(array.iter().any(|v| v["id"] == 3));
    }
}