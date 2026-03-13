use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_keys = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", path_str);
            continue;
        }

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_data: JsonValue = serde_json::from_str(&contents)?;

        match json_data {
            JsonValue::Array(arr) => {
                for item in arr {
                    if let Some(key) = extract_unique_key(&item) {
                        if seen_keys.contains_key(&key) {
                            eprintln!("Duplicate key {} found, skipping.", key);
                            continue;
                        }
                        seen_keys.insert(key.clone(), true);
                    }
                    merged_array.push(item);
                }
            }
            JsonValue::Object(obj) => {
                if let Some(key) = extract_unique_key(&JsonValue::Object(obj.clone())) {
                    if seen_keys.contains_key(&key) {
                        eprintln!("Duplicate key {} found, skipping.", key);
                        continue;
                    }
                    seen_keys.insert(key.clone(), true);
                }
                merged_array.push(JsonValue::Object(obj));
            }
            _ => {
                eprintln!("Warning: File {} does not contain JSON object or array, skipping.", path_str);
            }
        }
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &merged_array)?;

    println!("Successfully merged {} items into {}", merged_array.len(), output_path);
    Ok(())
}

fn extract_unique_key(value: &JsonValue) -> Option<String> {
    if let JsonValue::Object(map) = value {
        if let Some(id) = map.get("id") {
            if let Some(id_str) = id.as_str() {
                return Some(id_str.to_string());
            }
        }
        if let Some(name) = map.get("name") {
            if let Some(name_str) = name.as_str() {
                return Some(name_str.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        writeln!(file1.as_file(), r#"[{"id": "1", "value": "test1"}, {"id": "2", "value": "test2"}]"#).unwrap();

        let file2 = NamedTempFile::new().unwrap();
        writeln!(file2.as_file(), r#"[{"id": "3", "value": "test3"}]"#).unwrap();

        let output_file = NamedTempFile::new().unwrap();

        let paths = vec![file1.path().to_str().unwrap(), file2.path().to_str().unwrap()];
        let result = merge_json_files(&paths, output_file.path().to_str().unwrap());

        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: JsonValue = serde_json::from_str(&output_content).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }
}