
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;
use std::collections::HashSet;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    if paths.is_empty() {
        return Err("No input files provided".to_string());
    }

    let mut merged = Map::new();
    let mut processed_keys = HashSet::new();
    let mut conflict_log = Vec::new();

    for (idx, path) in paths.iter().enumerate() {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        
        let json: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if processed_keys.contains(&key) {
                    conflict_log.push(format!(
                        "Conflict: Key '{}' already exists from file {}, overwritten by file {}",
                        key,
                        paths[idx-1].as_ref().display(),
                        path.as_ref().display()
                    ));
                }
                merged.insert(key.clone(), value);
                processed_keys.insert(key);
            }
        } else {
            return Err(format!("File {} does not contain a JSON object", path.as_ref().display()));
        }
    }

    if !conflict_log.is_empty() {
        eprintln!("Merge conflicts detected:");
        for log in &conflict_log {
            eprintln!("  {}", log);
        }
    }

    Ok(Value::Object(merged))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_basic() {
        let file1 = create_temp_json(r#"{"a": 1, "b": 2}"#).unwrap();
        let file2 = create_temp_json(r#"{"c": 3, "d": 4}"#).unwrap();
        
        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let obj = result.as_object().unwrap();
        
        assert_eq!(obj.len(), 4);
        assert_eq!(obj["a"], 1);
        assert_eq!(obj["d"], 4);
    }

    #[test]
    fn test_merge_conflict() {
        let file1 = create_temp_json(r#"{"x": "first"}"#).unwrap();
        let file2 = create_temp_json(r#"{"x": "second"}"#).unwrap();
        
        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        assert_eq!(result["x"], "second");
    }

    fn create_temp_json(content: &str) -> std::io::Result<NamedTempFile> {
        let mut file = NamedTempFile::new()?;
        file.write_all(content.as_bytes())?;
        Ok(file)
    }
}use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
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
        writeln!(file2, r#"{"city": "Berlin", "age": 35}"#).unwrap();

        let result = merge_json_files(&[file1.path().to_str().unwrap(), file2.path().to_str().unwrap()]).unwrap();
        let result_obj = result.as_object().unwrap();

        assert_eq!(result_obj.get("name").unwrap().as_str().unwrap(), "Alice");
        assert_eq!(result_obj.get("city").unwrap().as_str().unwrap(), "Berlin");
        assert_eq!(result_obj.get("age").unwrap().as_u64().unwrap(), 35);
    }
}