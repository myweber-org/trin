
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), String> {
    if paths.is_empty() {
        return Err("No input files provided".to_string());
    }

    let mut merged: Map<String, Value> = Map::new();
    let mut conflict_log: Vec<String> = Vec::new();

    for (idx, path) in paths.iter().enumerate() {
        let content = fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        let json: Value = serde_json::from_str(&content).map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        if let Value::Object(obj) = json {
            merge_object(&mut merged, obj, &mut conflict_log, idx);
        } else {
            return Err(format!("File {} does not contain a JSON object", path.as_ref().display()));
        }
    }

    if !conflict_log.is_empty() {
        eprintln!("Conflicts detected during merge:");
        for log in &conflict_log {
            eprintln!("  {}", log);
        }
    }

    let output_value = Value::Object(merged);
    let output_str = serde_json::to_string_pretty(&output_value).map_err(|e| format!("Failed to serialize output: {}", e))?;
    fs::write(output_path, output_str).map_err(|e| format!("Failed to write output file: {}", e))?;

    Ok(())
}

fn merge_object(base: &mut Map<String, Value>, new: Map<String, Value>, conflict_log: &mut Vec<String>, file_index: usize) {
    for (key, new_value) in new {
        match base.get_mut(&key) {
            Some(existing_value) => {
                match (existing_value, new_value) {
                    (Value::Object(ref mut existing_obj), Value::Object(new_obj)) => {
                        merge_object(existing_obj, new_obj, conflict_log, file_index);
                    }
                    (Value::Array(ref mut existing_arr), Value::Array(new_arr)) => {
                        let existing_set: HashSet<_> = existing_arr.iter().collect();
                        for item in new_arr {
                            if !existing_set.contains(&item) {
                                existing_arr.push(item);
                            }
                        }
                    }
                    (existing, new) if existing == &new => {}
                    _ => {
                        conflict_log.push(format!("Key '{}' conflict: file {} overwrites previous value", key, file_index));
                        base.insert(key, new);
                    }
                }
            }
            None => {
                base.insert(key, new_value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_merge() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": {"x": 10}}"#).unwrap();
        fs::write(&file2, r#"{"c": 3, "b": {"y": 20}}"#).unwrap();

        merge_json_files(&[file1.path(), file2.path()], output.path()).unwrap();

        let content = fs::read_to_string(output.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        assert_eq!(parsed["a"], 1);
        assert_eq!(parsed["c"], 3);
        assert_eq!(parsed["b"]["x"], 10);
        assert_eq!(parsed["b"]["y"], 20);
    }
}