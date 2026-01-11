use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_objects = HashSet::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", file_path);
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    let serialized = serde_json::to_string(&item)?;
                    if seen_objects.insert(serialized) {
                        merged_array.push(item);
                    }
                }
            }
            Value::Object(_) => {
                let serialized = serde_json::to_string(&json_value)?;
                if seen_objects.insert(serialized) {
                    merged_array.push(json_value);
                }
            }
            _ => eprintln!("Warning: File {} does not contain JSON object or array, skipping.", file_path),
        }
    }

    let output_json = json!(merged_array);
    fs::write(output_path, serde_json::to_string_pretty(&output_json)?)?;
    println!("Merged JSON written to {}", output_path);
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

        fs::write(file1.path(), r#"[{"id": 1}, {"id": 2}]"#).unwrap();
        fs::write(file2.path(), r#"[{"id": 2}, {"id": 3}]"#).unwrap();

        let paths = vec![file1.path().to_str().unwrap(), file2.path().to_str().unwrap()];
        merge_json_files(&paths, output_file.path().to_str().unwrap()).unwrap();

        let content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }
}