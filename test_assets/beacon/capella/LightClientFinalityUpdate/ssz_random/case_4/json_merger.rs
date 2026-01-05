
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"name": "Alice", "age": 30}"#).unwrap();
        writeln!(file2, r#"{"city": "Berlin", "active": true}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]).unwrap();

        assert_eq!(result["name"], "Alice");
        assert_eq!(result["age"], 30);
        assert_eq!(result["city"], "Berlin");
        assert_eq!(result["active"], true);
    }

    #[test]
    fn test_merge_with_missing_file() {
        let mut file1 = NamedTempFile::new().unwrap();
        writeln!(file1, r#"{"data": "test"}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            "non_existent_file.json",
        ]).unwrap();

        assert_eq!(result["data"], "test");
        assert!(result.get("nonexistent").is_none());
    }
}
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde_json::{Map, Value};

pub fn merge_json_files(file_paths: &[impl AsRef<Path>]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path in file_paths {
        let file = File::open(path.as_ref())?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn merge_json_with_strategy(
    file_paths: &[impl AsRef<Path>],
    conflict_strategy: ConflictStrategy,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path in file_paths {
        let file = File::open(path.as_ref())?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                match conflict_strategy {
                    ConflictStrategy::Overwrite => {
                        merged_map.insert(key, value);
                    }
                    ConflictStrategy::Skip => {
                        merged_map.entry(key).or_insert(value);
                    }
                    ConflictStrategy::MergeObjects => {
                        if let Some(existing) = merged_map.get_mut(&key) {
                            if let (Value::Object(existing_map), Value::Object(new_map)) = (existing, &value) {
                                let mut combined = existing_map.clone();
                                for (k, v) in new_map {
                                    combined.insert(k.clone(), v.clone());
                                }
                                *existing = Value::Object(combined);
                            } else {
                                merged_map.insert(key, value);
                            }
                        } else {
                            merged_map.insert(key, value);
                        }
                    }
                }
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    let final_map: Map<String, Value> = merged_map.into_iter().collect();
    Ok(Value::Object(final_map))
}

#[derive(Debug, Clone, Copy)]
pub enum ConflictStrategy {
    Overwrite,
    Skip,
    MergeObjects,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_json(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn test_merge_json_files() {
        let file1 = create_temp_json(r#"{"a": 1, "b": 2}"#);
        let file2 = create_temp_json(r#"{"c": 3, "d": 4}"#);

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected: Value = serde_json::from_str(r#"{"a": 1, "b": 2, "c": 3, "d": 4}"#).unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_with_overwrite_strategy() {
        let file1 = create_temp_json(r#"{"a": 1, "b": 2}"#);
        let file2 = create_temp_json(r#"{"b": 99, "c": 3}"#);

        let result = merge_json_with_strategy(&[file1.path(), file2.path()], ConflictStrategy::Overwrite).unwrap();
        let expected: Value = serde_json::from_str(r#"{"a": 1, "b": 99, "c": 3}"#).unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_with_skip_strategy() {
        let file1 = create_temp_json(r#"{"a": 1, "b": 2}"#);
        let file2 = create_temp_json(r#"{"b": 99, "c": 3}"#);

        let result = merge_json_with_strategy(&[file1.path(), file2.path()], ConflictStrategy::Skip).unwrap();
        let expected: Value = serde_json::from_str(r#"{"a": 1, "b": 2, "c": 3}"#).unwrap();

        assert_eq!(result, expected);
    }
}