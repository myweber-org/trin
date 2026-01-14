use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in input_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", path_str);
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Key '{}' already exists, overwriting.", key);
                }
                merged_map.insert(key, value);
            }
        } else {
            eprintln!("Warning: File {} does not contain a JSON object, skipping.", path_str);
        }
    }

    let merged_json = Value::Object(merged_map);
    let output_content = serde_json::to_string_pretty(&merged_json)?;
    fs::write(output_path, output_content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"a": 1, "b": "test"}"#).unwrap();
        writeln!(file2, r#"{"c": true, "d": [1,2,3]}"#).unwrap();

        let inputs = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];
        let output_path = output_file.path().to_str().unwrap();

        let result = merge_json_files(&inputs, output_path);
        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_path).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();
        assert_eq!(parsed["a"], 1);
        assert_eq!(parsed["b"], "test");
        assert_eq!(parsed["c"], true);
        assert_eq!(parsed["d"], Value::from(vec![1, 2, 3]));
    }
}