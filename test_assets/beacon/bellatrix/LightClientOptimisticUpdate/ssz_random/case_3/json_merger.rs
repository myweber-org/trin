use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_keys = HashMap::new();

    for path in paths {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        let json_data: JsonValue = serde_json::from_str(&content)?;

        match json_data {
            JsonValue::Array(arr) => {
                for item in arr {
                    if let Some(key) = extract_unique_key(&item) {
                        if !seen_keys.contains_key(&key) {
                            seen_keys.insert(key.clone(), true);
                            merged_array.push(item);
                        }
                    } else {
                        merged_array.push(item);
                    }
                }
            }
            JsonValue::Object(_) => {
                if let Some(key) = extract_unique_key(&json_data) {
                    if !seen_keys.contains_key(&key) {
                        seen_keys.insert(key.clone(), true);
                        merged_array.push(json_data);
                    }
                } else {
                    merged_array.push(json_data);
                }
            }
            _ => return Err("Unsupported JSON structure: expected array or object".into()),
        }
    }

    let output_json = JsonValue::Array(merged_array);
    let output_str = serde_json::to_string_pretty(&output_json)?;
    fs::write(output_path, output_str)?;

    Ok(())
}

fn extract_unique_key(value: &JsonValue) -> Option<String> {
    if let JsonValue::Object(map) = value {
        if let Some(id) = map.get("id") {
            if let Some(id_str) = id.as_str() {
                return Some(id_str.to_string());
            }
        }
        if let Some(uuid) = map.get("uuid") {
            if let Some(uuid_str) = uuid.as_str() {
                return Some(uuid_str.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_arrays() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"[{"id": "a", "value": 1}, {"id": "b", "value": 2}]"#).unwrap();
        fs::write(&file2, r#"[{"id": "b", "value": 3}, {"id": "c", "value": 4}]"#).unwrap();

        let paths = [file1.path(), file2.path()];
        merge_json_files(&paths, output.path()).unwrap();

        let content = fs::read_to_string(output.path()).unwrap();
        let parsed: JsonValue = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }
}