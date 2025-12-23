
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::Path;

use serde_json::{Value, Map};

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map: Map<String, Value> = Map::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", input_path);
            continue;
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Key '{}' already exists, overwriting with value from {}.", key, input_path);
                }
                merged_map.insert(key, value);
            }
        } else {
            eprintln!("Warning: File {} does not contain a JSON object at root, skipping.", input_path);
        }
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &Value::Object(merged_map))?;

    println!("Successfully merged {} files into {}", input_paths.len(), output_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), r#"{"a": 1, "b": "test"}"#).unwrap();
        fs::write(file2.path(), r#"{"c": true, "d": [1,2,3]}"#).unwrap();

        let inputs = &[file1.path().to_str().unwrap(), file2.path().to_str().unwrap()];
        let output_path = output_file.path().to_str().unwrap();

        let result = merge_json_files(inputs, output_path);
        assert!(result.is_ok());

        let content = fs::read_to_string(output_path).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert!(parsed.get("a").is_some());
        assert!(parsed.get("b").is_some());
        assert!(parsed.get("c").is_some());
        assert!(parsed.get("d").is_some());
    }
}