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

        if let Value::Object(map) = json_value {
            for (key, value) in map {
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
    match strategy {
        MergeStrategy::Overwrite => merge_json_files(file_paths),
        MergeStrategy::CombineArrays => merge_with_array_combination(file_paths),
    }
}

fn merge_with_array_combination(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut combined_map: HashMap<String, Vec<Value>> = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                match value {
                    Value::Array(arr) => {
                        combined_map
                            .entry(key)
                            .or_insert_with(Vec::new)
                            .extend(arr);
                    }
                    _ => {
                        if !combined_map.contains_key(&key) {
                            let mut vec = Vec::new();
                            vec.push(value);
                            combined_map.insert(key, vec);
                        }
                    }
                }
            }
        }
    }

    let mut result_map = Map::new();
    for (key, values) in combined_map {
        if values.len() == 1 {
            result_map.insert(key, values.into_iter().next().unwrap());
        } else {
            result_map.insert(key, Value::Array(values));
        }
    }

    Ok(Value::Object(result_map))
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
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        let json1 = json!({
            "name": "Alice",
            "age": 30
        });

        let json2 = json!({
            "city": "Wonderland",
            "age": 31
        });

        write!(file1, "{}", json1.to_string()).unwrap();
        write!(file2, "{}", json2.to_string()).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ])
        .unwrap();

        assert_eq!(result["name"], "Alice");
        assert_eq!(result["age"], 31);
        assert_eq!(result["city"], "Wonderland");
    }

    #[test]
    fn test_merge_with_array_combination() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        let json1 = json!({
            "tags": ["rust", "json"],
            "count": 5
        });

        let json2 = json!({
            "tags": ["merge", "utility"],
            "count": 10
        });

        write!(file1, "{}", json1.to_string()).unwrap();
        write!(file2, "{}", json2.to_string()).unwrap();

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
        assert!(tags.contains(&json!("rust")));
        assert!(tags.contains(&json!("merge")));

        let count_array = result["count"].as_array().unwrap();
        assert_eq!(count_array.len(), 2);
    }
}