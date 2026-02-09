
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::Path;

use serde_json::{Value, json};

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_ids = HashMap::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", input_path);
            continue;
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                        if seen_ids.contains_key(id) {
                            eprintln!("Duplicate ID '{}' found, skipping.", id);
                            continue;
                        }
                        seen_ids.insert(id.to_string(), ());
                    }
                    merged_array.push(item);
                }
            }
            Value::Object(obj) => {
                if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                    if seen_ids.contains_key(id) {
                        eprintln!("Duplicate ID '{}' found, skipping.", id);
                        continue;
                    }
                    seen_ids.insert(id.to_string(), ());
                }
                merged_array.push(json!(obj));
            }
            _ => {
                eprintln!("Warning: {} contains neither array nor object, skipping.", input_path);
            }
        }
    }

    let output_value = json!(merged_array);
    let mut output_file = File::create(output_path)?;
    let pretty_json = serde_json::to_string_pretty(&output_value)?;
    output_file.write_all(pretty_json.as_bytes())?;

    println!("Successfully merged {} items into {}", merged_array.len(), output_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let json1 = r#"[{"id": "1", "name": "Alice"}, {"id": "2", "name": "Bob"}]"#;
        let json2 = r#"{"id": "3", "name": "Charlie"}"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), json1).unwrap();
        fs::write(file2.path(), json2).unwrap();

        let inputs = &[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(inputs, output_file.path().to_str().unwrap());
        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }
}