use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::Path;

use serde_json::{Value, json};

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
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
                eprintln!("Warning: File {} does not contain JSON array or object, skipping.", path_str);
            }
        }
    }

    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    serde_json::to_writer_pretty(writer, &json!(merged_array))?;

    println!("Successfully merged {} files into {}", input_paths.len(), output_path);
    Ok(())
}

pub fn merge_json_with_deduplication(
    input_paths: &[&str], 
    output_path: &str, 
    unique_key: &str
) -> Result<(), Box<dyn std::error::Error>> {
    let mut unique_map: HashMap<String, Value> = HashMap::new();

    for path_str in input_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            eprintln!("Warning: File {} does not exist, skipping.", path_str);
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Array(arr) = json_value {
            for item in arr {
                if let Value::Object(map) = item {
                    if let Some(Value::String(key_value)) = map.get(unique_key) {
                        unique_map.insert(key_value.clone(), Value::Object(map));
                    }
                }
            }
        }
    }

    let deduplicated: Vec<Value> = unique_map.into_values().collect();
    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    serde_json::to_writer_pretty(writer, &json!(deduplicated))?;

    println!("Successfully merged and deduplicated {} unique items into {}", deduplicated.len(), output_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use serde_json::json;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        let data1 = json!([{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]);
        let data2 = json!([{"id": 3, "name": "Charlie"}]);

        fs::write(file1.path(), serde_json::to_string_pretty(&data1).unwrap()).unwrap();
        fs::write(file2.path(), serde_json::to_string_pretty(&data2).unwrap()).unwrap();

        let inputs = &[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        merge_json_files(inputs, output_file.path().to_str().unwrap()).unwrap();

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();

        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_merge_with_deduplication() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        let data1 = json!([{"id": "1", "name": "Alice"}, {"id": "2", "name": "Bob"}]);
        let data2 = json!([{"id": "1", "name": "Alice Duplicate"}, {"id": "3", "name": "Charlie"}]);

        fs::write(file1.path(), serde_json::to_string_pretty(&data1).unwrap()).unwrap();
        fs::write(file2.path(), serde_json::to_string_pretty(&data2).unwrap()).unwrap();

        let inputs = &[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        merge_json_with_deduplication(inputs, output_file.path().to_str().unwrap(), "id").unwrap();

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();

        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }
}