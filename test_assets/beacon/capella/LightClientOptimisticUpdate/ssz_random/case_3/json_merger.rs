use serde_json::{Map, Value};
use std::collections::HashMap;
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

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn merge_json_with_strategy(
    file_paths: &[&str],
    strategy: MergeStrategy,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut accumulator: HashMap<String, Value> = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                match strategy {
                    MergeStrategy::Overwrite => {
                        accumulator.insert(key, value);
                    }
                    MergeStrategy::CombineArrays => {
                        if let Some(existing) = accumulator.get(&key) {
                            if existing.is_array() && value.is_array() {
                                let mut combined = existing.as_array().unwrap().clone();
                                combined.extend(value.as_array().unwrap().clone());
                                accumulator.insert(key, Value::Array(combined));
                            } else {
                                accumulator.insert(key, value);
                            }
                        } else {
                            accumulator.insert(key, value);
                        }
                    }
                }
            }
        }
    }

    let map: Map<String, Value> = accumulator.into_iter().collect();
    Ok(Value::Object(map))
}

#[derive(Debug, Clone, Copy)]
pub enum MergeStrategy {
    Overwrite,
    CombineArrays,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_merge() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        let json1 = json!({"name": "Alice", "age": 30});
        let json2 = json!({"city": "Berlin", "active": true});

        write!(file1, "{}", json1).unwrap();
        write!(file2, "{}", json2).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ])
        .unwrap();

        assert_eq!(result["name"], "Alice");
        assert_eq!(result["age"], 30);
        assert_eq!(result["city"], "Berlin");
        assert_eq!(result["active"], true);
    }

    #[test]
    fn test_combine_arrays_strategy() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        let json1 = json!({"tags": ["rust", "json"], "id": 1});
        let json2 = json!({"tags": ["merge", "utility"], "extra": "data"});

        write!(file1, "{}", json1).unwrap();
        write!(file2, "{}", json2).unwrap();

        let result = merge_json_with_strategy(
            &[
                file1.path().to_str().unwrap(),
                file2.path().to_str().unwrap(),
            ],
            MergeStrategy::CombineArrays,
        )
        .unwrap();

        let tags = result["tags"].as_array().unwrap();
        assert_eq!(tags.len(), 4);
        assert!(tags.contains(&"rust".into()));
        assert!(tags.contains(&"merge".into()));
        assert_eq!(result["id"], 1);
        assert_eq!(result["extra"], "data");
    }
}