use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str).into());
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        if let serde_json::Value::Object(obj) = json_value {
            for (key, value) in obj {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    Ok(serde_json::Value::Object(
        merged_map
            .into_iter()
            .collect::<serde_json::Map<String, serde_json::Value>>(),
    ))
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

        writeln!(file1, r#"{{ "name": "Alice", "age": 30 }}"#).unwrap();
        writeln!(file2, r#"{{ "city": "London", "active": true }}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]);

        assert!(result.is_ok());
        let merged = result.unwrap();
        assert_eq!(merged["name"], "Alice");
        assert_eq!(merged["age"], 30);
        assert_eq!(merged["city"], "London");
        assert_eq!(merged["active"], true);
    }

    #[test]
    fn test_merge_with_conflict() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{{ "key": "first" }}"#).unwrap();
        writeln!(file2, r#"{{ "key": "second" }}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]);

        assert!(result.is_ok());
        let merged = result.unwrap();
        assert_eq!(merged["key"], "second");
    }
}