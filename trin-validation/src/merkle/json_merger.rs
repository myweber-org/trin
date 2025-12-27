
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[&str]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_data: JsonValue = serde_json::from_str(&contents)?;

        if let JsonValue::Object(map) = json_data {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    let merged_json = JsonValue::Object(
        merged_map
            .into_iter()
            .map(|(k, v)| (k, v))
            .collect()
    );

    Ok(merged_json)
}

pub fn write_merged_json(output_path: &str, json_value: &JsonValue) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(output_path)?;
    let json_string = serde_json::to_string_pretty(json_value)?;
    file.write_all(json_string.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_merge_json_files() {
        let temp_dir = TempDir::new().unwrap();
        let file1_path = temp_dir.path().join("data1.json");
        let file2_path = temp_dir.path().join("data2.json");

        fs::write(&file1_path, r#"{"name": "Alice", "age": 30}"#).unwrap();
        fs::write(&file2_path, r#"{"city": "London", "active": true}"#).unwrap();

        let paths = vec![
            file1_path.to_str().unwrap(),
            file2_path.to_str().unwrap(),
        ];

        let merged = merge_json_files(&paths).unwrap();
        assert_eq!(merged["name"], "Alice");
        assert_eq!(merged["age"], 30);
        assert_eq!(merged["city"], "London");
        assert_eq!(merged["active"], true);
    }
}use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str).into());
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
            return Err("Top-level JSON value must be an object".into());
        }
    }

    Ok(Value::Object(merged_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        let json1 = json!({
            "name": "Alice",
            "age": 30
        });
        let json2 = json!({
            "city": "London",
            "age": 31
        });

        write!(file1, "{}", json1.to_string()).unwrap();
        write!(file2, "{}", json2.to_string()).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let expected = json!({
            "name": "Alice",
            "age": 31,
            "city": "London"
        });

        assert_eq!(result, expected);
    }
}