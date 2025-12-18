use serde_json::{Value, json};
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::Path;

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

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
                merged_array.extend(arr);
            }
            Value::Object(_) => {
                merged_array.push(json_value);
            }
            _ => {
                eprintln!("Warning: File {} does not contain a JSON object or array, skipping.", input_path);
            }
        }
    }

    let output_file = File::create(output_path)?;
    let merged_json = json!(merged_array);
    serde_json::to_writer_pretty(output_file, &merged_json)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let json1 = r#"[{"id": 1}, {"id": 2}]"#;
        let json2 = r#"{"id": 3}"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), json1).unwrap();
        fs::write(file2.path(), json2).unwrap();

        let input_paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        merge_json_files(&input_paths, output_file.path().to_str().unwrap()).unwrap();

        let content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        assert!(parsed.is_array());
        let array = parsed.as_array().unwrap();
        assert_eq!(array.len(), 3);
        assert_eq!(array[0]["id"], 1);
        assert_eq!(array[1]["id"], 2);
        assert_eq!(array[2]["id"], 3);
    }
}