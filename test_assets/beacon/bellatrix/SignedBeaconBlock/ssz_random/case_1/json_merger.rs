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
            return Err("Top-level JSON must be an object".into());
        }
    }

    Ok(serde_json::Value::Object(merged_map))
}use serde_json::{Value, Map};
use std::collections::HashSet;

pub fn merge_json(base: &mut Value, extension: &Value, overwrite_arrays: bool) {
    match (base, extension) {
        (Value::Object(base_map), Value::Object(ext_map)) => {
            for (key, ext_value) in ext_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    merge_json(base_value, ext_value, overwrite_arrays);
                } else {
                    base_map.insert(key.clone(), ext_value.clone());
                }
            }
        }
        (Value::Array(base_arr), Value::Array(ext_arr)) => {
            if overwrite_arrays {
                *base_arr = ext_arr.clone();
            } else {
                let mut seen = HashSet::new();
                for item in base_arr.iter() {
                    if let Value::Object(map) = item {
                        if let Some(Value::String(id)) = map.get("id") {
                            seen.insert(id.clone());
                        }
                    }
                }
                
                for item in ext_arr {
                    if let Value::Object(map) = item {
                        if let Some(Value::String(id)) = map.get("id") {
                            if !seen.contains(id) {
                                base_arr.push(item.clone());
                                seen.insert(id.clone());
                            }
                        } else {
                            base_arr.push(item.clone());
                        }
                    } else {
                        base_arr.push(item.clone());
                    }
                }
            }
        }
        (base, extension) => {
            if !overwrite_arrays || !extension.is_array() {
                *base = extension.clone();
            }
        }
    }
}

pub fn merge_json_with_strategy(
    base: &Value,
    extension: &Value,
    strategy: MergeStrategy,
) -> Value {
    let mut result = base.clone();
    merge_json(&mut result, extension, strategy.overwrite_arrays());
    result
}

#[derive(Debug, Clone, Copy)]
pub enum MergeStrategy {
    Shallow,
    Deep,
    DeepPreserveArrays,
}

impl MergeStrategy {
    fn overwrite_arrays(&self) -> bool {
        match self {
            MergeStrategy::Shallow => true,
            MergeStrategy::Deep => true,
            MergeStrategy::DeepPreserveArrays => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_shallow_merge() {
        let base = json!({"a": 1, "b": {"inner": 2}});
        let ext = json!({"b": {"new": 3}, "c": 4});
        let result = merge_json_with_strategy(&base, &ext, MergeStrategy::Shallow);
        
        assert_eq!(result["a"], 1);
        assert_eq!(result["b"], json!({"new": 3}));
        assert_eq!(result["c"], 4);
    }

    #[test]
    fn test_deep_merge_preserve_arrays() {
        let base = json!({
            "items": [{"id": "1", "name": "first"}],
            "config": {"timeout": 30}
        });
        let ext = json!({
            "items": [{"id": "2", "name": "second"}],
            "config": {"retries": 3}
        });
        
        let result = merge_json_with_strategy(&base, &ext, MergeStrategy::DeepPreserveArrays);
        let items = result["items"].as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(result["config"]["timeout"], 30);
        assert_eq!(result["config"]["retries"], 3);
    }
}