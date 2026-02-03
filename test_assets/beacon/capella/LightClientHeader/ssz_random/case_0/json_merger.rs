
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufReader, Write};
use std::path::Path;

use serde_json::{Map, Value};

pub fn merge_json_files<P: AsRef<Path>>(input_paths: &[P], output_path: P) -> io::Result<()> {
    let mut merged_map = Map::new();

    for path in input_paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "JSON root must be an object",
            ));
        }
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &Value::Object(merged_map))
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    Ok(())
}

pub fn merge_json_with_strategy<P: AsRef<Path>>(
    input_paths: &[P],
    output_path: P,
    conflict_strategy: ConflictStrategy,
) -> io::Result<()> {
    let mut merged: HashMap<String, Value> = HashMap::new();

    for path in input_paths {
        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                match conflict_strategy {
                    ConflictStrategy::Overwrite => {
                        merged.insert(key, value);
                    }
                    ConflictStrategy::Skip => {
                        merged.entry(key).or_insert(value);
                    }
                    ConflictStrategy::MergeObjects => {
                        if let Some(existing) = merged.get_mut(&key) {
                            if existing.is_object() && value.is_object() {
                                merge_json_objects(existing, &value);
                            } else {
                                merged.insert(key, value);
                            }
                        } else {
                            merged.insert(key, value);
                        }
                    }
                }
            }
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "JSON root must be an object",
            ));
        }
    }

    let output_value: Value = serde_json::to_value(&merged)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &output_value)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    Ok(())
}

fn merge_json_objects(target: &mut Value, source: &Value) {
    if let (Value::Object(target_map), Value::Object(source_map)) = (target, source) {
        for (key, value) in source_map {
            if let Some(existing) = target_map.get_mut(key) {
                if existing.is_object() && value.is_object() {
                    merge_json_objects(existing, value);
                } else {
                    target_map.insert(key.clone(), value.clone());
                }
            } else {
                target_map.insert(key.clone(), value.clone());
            }
        }
    }
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
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_merge() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": 2}"#).unwrap();
        fs::write(&file2, r#"{"c": 3, "d": 4}"#).unwrap();

        merge_json_files(&[file1.path(), file2.path()], output.path()).unwrap();

        let content = fs::read_to_string(output.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["a"], 1);
        assert_eq!(parsed["b"], 2);
        assert_eq!(parsed["c"], 3);
        assert_eq!(parsed["d"], 4);
    }
}