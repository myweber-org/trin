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
                strategy.apply(&mut accumulator, key, value);
            }
        }
    }

    let final_map: Map<String, Value> = accumulator.into_iter().collect();
    Ok(Value::Object(final_map))
}

pub enum MergeStrategy {
    Overwrite,
    KeepFirst,
    MergeObjects,
}

impl MergeStrategy {
    fn apply(&self, acc: &mut HashMap<String, Value>, key: String, value: Value) {
        match self {
            MergeStrategy::Overwrite => {
                acc.insert(key, value);
            }
            MergeStrategy::KeepFirst => {
                acc.entry(key).or_insert(value);
            }
            MergeStrategy::MergeObjects => {
                if let Some(existing) = acc.get_mut(&key) {
                    if let (Value::Object(existing_obj), Value::Object(new_obj)) =
                        (existing, &value)
                    {
                        let mut merged = existing_obj.clone();
                        for (k, v) in new_obj {
                            merged.insert(k.clone(), v.clone());
                        }
                        *existing = Value::Object(merged);
                    } else {
                        acc.insert(key, value);
                    }
                } else {
                    acc.insert(key, value);
                }
            }
        }
    }
}