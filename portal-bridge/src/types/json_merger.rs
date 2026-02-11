
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();
    let mut seen_keys = HashSet::new();

    for &file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path).into());
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                if seen_keys.contains(&key) {
                    return Err(format!("Duplicate key '{}' found in files", key).into());
                }
                seen_keys.insert(key.clone());
                merged_map.insert(key, value);
            }
        } else {
            return Err("JSON root is not an object".into());
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

        let data1 = json!({"name": "Alice", "age": 30});
        let data2 = json!({"city": "Berlin", "country": "Germany"});

        write!(file1, "{}", data1.to_string()).unwrap();
        write!(file2, "{}", data2.to_string()).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let expected = json!({
            "name": "Alice",
            "age": 30,
            "city": "Berlin",
            "country": "Germany"
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_duplicate_key_error() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        let data1 = json!({"key": "value1"});
        let data2 = json!({"key": "value2"});

        write!(file1, "{}", data1.to_string()).unwrap();
        write!(file2, "{}", data2.to_string()).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate key"));
    }
}
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(Value::Object(merged_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_merge_json_files() {
        let dir = tempdir().unwrap();
        let file1_path = dir.path().join("data1.json");
        let file2_path = dir.path().join("data2.json");

        let mut file1 = File::create(&file1_path).unwrap();
        file1.write_all(b"{\"name\": \"Alice\", \"age\": 30}").unwrap();

        let mut file2 = File::create(&file2_path).unwrap();
        file2.write_all(b"{\"city\": \"Berlin\", \"active\": true}").unwrap();

        let paths = vec![
            file1_path.to_str().unwrap(),
            file2_path.to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "Alice");
        assert_eq!(obj.get("age").unwrap().as_i64().unwrap(), 30);
        assert_eq!(obj.get("city").unwrap().as_str().unwrap(), "Berlin");
        assert!(obj.get("active").unwrap().as_bool().unwrap());
    }
}