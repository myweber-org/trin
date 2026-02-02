
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;
use serde_json::{Value, json};

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", file_path);
            continue;
        }

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: Value = serde_json::from_str(&contents)?;
        merged_array.push(json_value);
    }

    let output_json = json!(merged_array);
    let output_str = serde_json::to_string_pretty(&output_json)?;
    fs::write(output_path, output_str)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_merge_json_files() {
        let json1 = r#"{"id": 1, "name": "Alice"}"#;
        let json2 = r#"{"id": 2, "name": "Bob"}"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        writeln!(&file1, "{}", json1).unwrap();
        writeln!(&file2, "{}", json2).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths, output_file.path().to_str().unwrap());
        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 2);
    }
}use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", file_path);
            continue;
        }

        let content = fs::read_to_string(path)?;
        let parsed: Value = serde_json::from_str(&content)?;

        if parsed.is_array() {
            if let Value::Array(arr) = parsed {
                merged_array.extend(arr);
            }
        } else {
            merged_array.push(parsed);
        }
    }

    let output_json = json!(merged_array);
    fs::write(output_path, output_json.to_string())?;

    println!("Successfully merged {} files into {}", file_paths.len(), output_path);
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
        fs::write(file2.path(), r#"{"name": "test"}"#).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths, output_file.path().to_str().unwrap());
        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }
}