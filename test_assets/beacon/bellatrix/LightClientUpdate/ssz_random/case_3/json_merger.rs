use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::Path;

use serde_json::{Value, json};

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        if !path.exists() {
            eprintln!("Warning: File {} does not exist, skipping.", input_path);
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
                eprintln!("Warning: File {} does not contain a JSON object or array, skipping.", input_path);
            }
        }
    }

    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    serde_json::to_writer_pretty(writer, &json!(merged_array))?;

    Ok(())
}

pub fn merge_json_with_deduplication(
    input_paths: &[&str], 
    output_path: &str, 
    unique_key: &str
) -> Result<(), Box<dyn std::error::Error>> {
    let mut unique_map: HashMap<String, Value> = HashMap::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        if !path.exists() {
            eprintln!("Warning: File {} does not exist, skipping.", input_path);
            continue;
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        let items = match json_value {
            Value::Array(arr) => arr,
            Value::Object(_) => vec![json_value],
            _ => {
                eprintln!("Warning: File {} contains invalid JSON structure, skipping.", input_path);
                continue;
            }
        };

        for item in items {
            if let Value::Object(map) = item {
                if let Some(key_value) = map.get(unique_key) {
                    let key_string = key_value.to_string();
                    unique_map.insert(key_string, Value::Object(map));
                }
            }
        }
    }

    let deduplicated_array: Vec<Value> = unique_map.into_values().collect();
    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    serde_json::to_writer_pretty(writer, &json!(deduplicated_array))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        writeln!(file1.as_file(), r#"[{{"id": 1}}, {{"id": 2}}]"#).unwrap();
        writeln!(file2.as_file(), r#"{{"id": 3}}"#).unwrap();

        let inputs = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&inputs, output_file.path().to_str().unwrap());
        assert!(result.is_ok());

        let content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_merge_with_deduplication() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        writeln!(file1.as_file(), r#"[{{"id": 1, "name": "Alice"}}, {{"id": 2, "name": "Bob"}}]"#).unwrap();
        writeln!(file2.as_file(), r#"[{{"id": 1, "name": "Alice Updated"}}, {{"id": 3, "name": "Charlie"}}]"#).unwrap();

        let inputs = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_with_deduplication(
            &inputs, 
            output_file.path().to_str().unwrap(), 
            "id"
        );
        assert!(result.is_ok());

        let content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        let array = parsed.as_array().unwrap();
        assert_eq!(array.len(), 3);
        
        let ids: Vec<i64> = array.iter()
            .filter_map(|v| v.get("id").and_then(|id| id.as_i64()))
            .collect();
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
        assert!(ids.contains(&3));
    }
}