
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged = Map::new();
    
    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;
        
        if let Value::Object(obj) = json {
            merge_objects(&mut merged, obj);
        }
    }
    
    let output_json = Value::Object(merged);
    let serialized = serde_json::to_string_pretty(&output_json)?;
    fs::write(output_path, serialized)?;
    
    Ok(())
}

fn merge_objects(base: &mut Map<String, Value>, new: Map<String, Value>) {
    for (key, new_value) in new {
        if let Some(existing_value) = base.get_mut(&key) {
            match (existing_value, new_value) {
                (Value::Object(ref mut base_obj), Value::Object(new_obj)) => {
                    merge_objects(base_obj, new_obj);
                }
                (Value::Array(ref mut base_arr), Value::Array(new_arr)) => {
                    base_arr.extend(new_arr);
                    base_arr.sort();
                    base_arr.dedup();
                }
                _ => {
                    *existing_value = new_value;
                }
            }
        } else {
            base.insert(key, new_value);
        }
    }
}use serde_json::{Map, Value};
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

pub fn merge_with_strategy(
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
                    MergeStrategy::SkipExisting => {
                        accumulator.entry(key).or_insert(value);
                    }
                    MergeStrategy::MergeObjects => {
                        if let Some(Value::Object(existing_obj)) = accumulator.get(&key) {
                            if let Value::Object(new_obj) = &value {
                                let mut combined = existing_obj.clone();
                                for (k, v) in new_obj {
                                    combined.insert(k.clone(), v.clone());
                                }
                                accumulator.insert(key, Value::Object(combined));
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

    let final_map: Map<String, Value> = accumulator.into_iter().collect();
    Ok(Value::Object(final_map))
}

#[derive(Debug, Clone, Copy)]
pub enum MergeStrategy {
    Overwrite,
    SkipExisting,
    MergeObjects,
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

        file1.write_all(b"{\"a\": 1, \"b\": 2}").unwrap();
        file2.write_all(b"{\"c\": 3, \"d\": 4}").unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];
        let result = merge_json_files(&paths).unwrap();

        assert_eq!(result["a"], 1);
        assert_eq!(result["b"], 2);
        assert_eq!(result["c"], 3);
        assert_eq!(result["d"], 4);
    }

    #[test]
    fn test_overwrite_strategy() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        file1.write_all(b"{\"key\": \"first\"}").unwrap();
        file2.write_all(b"{\"key\": \"second\"}").unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];
        let result = merge_with_strategy(&paths, MergeStrategy::Overwrite).unwrap();

        assert_eq!(result["key"], "second");
    }

    #[test]
    fn test_skip_existing_strategy() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        file1.write_all(b"{\"key\": \"first\"}").unwrap();
        file2.write_all(b"{\"key\": \"second\", \"other\": \"value\"}").unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];
        let result = merge_with_strategy(&paths, MergeStrategy::SkipExisting).unwrap();

        assert_eq!(result["key"], "first");
        assert_eq!(result["other"], "value");
    }
}