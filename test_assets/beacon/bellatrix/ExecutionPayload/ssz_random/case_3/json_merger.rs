use serde_json::{Value, Map};
use std::fs;
use std::path::Path;
use std::collections::HashSet;

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array: Vec<Value> = Vec::new();
    let mut seen_objects: HashSet<String> = HashSet::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        if !path.exists() {
            eprintln!("Warning: File {} does not exist, skipping.", input_path);
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    let serialized = item.to_string();
                    if !seen_objects.contains(&serialized) {
                        seen_objects.insert(serialized.clone());
                        merged_array.push(item);
                    }
                }
            }
            Value::Object(obj) => {
                let serialized = Value::Object(obj).to_string();
                if !seen_objects.contains(&serialized) {
                    seen_objects.insert(serialized.clone());
                    merged_array.push(Value::Object(Map::new()));
                }
            }
            _ => {
                eprintln!("Warning: JSON root in {} is neither array nor object, skipping.", input_path);
            }
        }
    }

    let output_value = Value::Array(merged_array);
    let output_json = serde_json::to_string_pretty(&output_value)?;
    fs::write(output_path, output_json)?;

    println!("Successfully merged {} files into {}", input_paths.len(), output_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_arrays() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        writeln!(file1, r#"[{"id": 1}, {"id": 2}]"#).unwrap();
        writeln!(file2, r#"[{"id": 2}, {"id": 3}]"#).unwrap();

        let inputs = [file1.path().to_str().unwrap(), file2.path().to_str().unwrap()];
        let output_path = output_file.path().to_str().unwrap();

        merge_json_files(&inputs, output_path).unwrap();

        let content = fs::read_to_string(output_path).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert!(parsed.is_array());
        let arr = parsed.as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }
}