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
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        let json1 = json!({
            "name": "test",
            "value": 42
        });

        let json2 = json!({
            "enabled": true,
            "tags": ["rust", "json"]
        });

        write!(file1, "{}", json1.to_string()).unwrap();
        write!(file2, "{}", json2.to_string()).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]).unwrap();

        assert_eq!(result["name"], "test");
        assert_eq!(result["value"], 42);
        assert_eq!(result["enabled"], true);
        assert!(result["tags"].is_array());
    }
}
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

        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Key '{}' already exists, overwriting.", key);
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object at the top level".into());
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
        writeln!(file2, r#"{"city": "London", "age": 35}"#).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "Alice");
        assert_eq!(obj.get("city").unwrap().as_str().unwrap(), "London");
        assert_eq!(obj.get("age").unwrap().as_u64().unwrap(), 35);
    }

    #[test]
    fn test_file_not_found() {
        let result = merge_json_files(&["nonexistent.json"]);
        assert!(result.is_err());
    }
}
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged = Map::new();
    let mut conflict_log = Vec::new();

    for path in paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json: Map<String, Value> = serde_json::from_reader(reader)?;

        for (key, value) in json {
            if merged.contains_key(&key) {
                let existing = merged.get(&key).unwrap();
                if existing != &value {
                    conflict_log.push(format!(
                        "Conflict for key '{}': existing {:?}, new {:?}",
                        key, existing, value
                    ));
                    merged.insert(format!("{}_conflict", key), value);
                }
            } else {
                merged.insert(key, value);
            }
        }
    }

    let output = File::create(output_path)?;
    serde_json::to_writer_pretty(output, &Value::Object(merged))?;

    if !conflict_log.is_empty() {
        let mut log_file = File::create("merge_conflicts.log")?;
        for entry in conflict_log {
            writeln!(log_file, "{}", entry)?;
        }
    }

    Ok(())
}

pub fn find_unique_keys<P: AsRef<Path>>(paths: &[P]) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
    let mut all_keys = HashSet::new();
    let mut common_keys = HashSet::new();
    let mut first = true;

    for path in paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json: Map<String, Value> = serde_json::from_reader(reader)?;
        let keys: HashSet<String> = json.keys().cloned().collect();

        if first {
            common_keys = keys.clone();
            first = false;
        } else {
            common_keys = common_keys.intersection(&keys).cloned().collect();
        }
        all_keys.extend(keys);
    }

    let unique_keys: HashSet<String> = all_keys.difference(&common_keys).cloned().collect();
    Ok(unique_keys)
}