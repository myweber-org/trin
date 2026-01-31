
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path).into());
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    Ok(serde_json::Value::Object(merged_map))
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

        writeln!(file1, r#"{"name": "Alice", "age": 30}"#).unwrap();
        writeln!(file2, r#"{"city": "London", "active": true}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]).unwrap();

        assert_eq!(result["name"], "Alice");
        assert_eq!(result["age"], 30);
        assert_eq!(result["city"], "London");
        assert_eq!(result["active"], true);
    }

    #[test]
    fn test_merge_with_conflict() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"id": 1, "value": "first"}"#).unwrap();
        writeln!(file2, r#"{"id": 2, "value": "second"}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]).unwrap();

        assert_eq!(result["id"], 2);
        assert_eq!(result["value"], "second");
    }
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;
use std::collections::HashSet;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged: Map<String, Value> = Map::new();
    let mut conflict_log: Vec<String> = Vec::new();
    let mut processed_keys: HashSet<String> = HashSet::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;
        
        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if processed_keys.contains(&key) {
                    conflict_log.push(format!("Conflict detected for key '{}' in file: {:?}", key, path.as_ref()));
                    continue;
                }
                merged.insert(key.clone(), value);
                processed_keys.insert(key);
            }
        }
    }

    let output_json = Value::Object(merged);
    let pretty_json = serde_json::to_string_pretty(&output_json)?;
    fs::write(output_path, pretty_json)?;

    if !conflict_log.is_empty() {
        let log_content = conflict_log.join("\n");
        fs::write("merge_conflicts.log", log_content)?;
        eprintln!("Merged with conflicts. See merge_conflicts.log for details.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_basic() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": 2}"#).unwrap();
        fs::write(&file2, r#"{"c": 3, "d": 4}"#).unwrap();

        merge_json_files(&[&file1, &file2], &output).unwrap();
        let content = fs::read_to_string(output).unwrap();
        assert!(content.contains("\"a\": 1"));
        assert!(content.contains("\"d\": 4"));
    }
}