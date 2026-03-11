
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged = Map::new();
    let mut conflict_log = Vec::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if merged.contains_key(&key) {
                    let existing = merged.get(&key).unwrap();
                    if existing != &value {
                        conflict_log.push(format!("Conflict for key '{}': {:?} vs {:?}", key, existing, value));
                        merged.insert(key, Value::Array(vec![existing.clone(), value]));
                    }
                } else {
                    merged.insert(key, value);
                }
            }
        }
    }

    let result = Value::Object(merged);
    let output = serde_json::to_string_pretty(&result)?;
    fs::write(output_path, output)?;

    if !conflict_log.is_empty() {
        let log_content = conflict_log.join("\n");
        fs::write("merge_conflicts.log", log_content)?;
        eprintln!("Conflicts detected. See merge_conflicts.log for details.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": "test"}"#).unwrap();
        fs::write(&file2, r#"{"b": "test", "c": true}"#).unwrap();

        merge_json_files(&[file1.path(), file2.path()], output.path()).unwrap();

        let content = fs::read_to_string(output.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        assert_eq!(parsed["a"], 1);
        assert_eq!(parsed["b"], "test");
        assert_eq!(parsed["c"], true);
    }
}