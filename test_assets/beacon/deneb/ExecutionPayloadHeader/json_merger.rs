
use serde_json::{Map, Value};
use std::collections::HashSet;

pub fn merge_json(base: &mut Value, update: &Value, strategy: MergeStrategy) -> Result<(), String> {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (key, update_value) in update_map {
                if base_map.contains_key(key) {
                    let base_value = base_map.get_mut(key).unwrap();
                    match strategy {
                        MergeStrategy::Overwrite => *base_value = update_value.clone(),
                        MergeStrategy::Recursive => {
                            merge_json(base_value, update_value, strategy.clone())?;
                        }
                        MergeStrategy::CombineArrays => {
                            combine_arrays(base_value, update_value)?;
                        }
                        MergeStrategy::Skip => continue,
                    }
                } else {
                    base_map.insert(key.clone(), update_value.clone());
                }
            }
            Ok(())
        }
        _ => Err("Both values must be JSON objects".to_string()),
    }
}

fn combine_arrays(base: &mut Value, update: &Value) -> Result<(), String> {
    if let (Value::Array(base_arr), Value::Array(update_arr)) = (base, update) {
        let mut seen = HashSet::new();
        for item in base_arr.iter() {
            if let Ok(serialized) = serde_json::to_string(item) {
                seen.insert(serialized);
            }
        }
        
        for item in update_arr {
            if let Ok(serialized) = serde_json::to_string(item) {
                if !seen.contains(&serialized) {
                    base_arr.push(item.clone());
                    seen.insert(serialized);
                }
            }
        }
        Ok(())
    } else {
        Err("Both values must be arrays for combine operation".to_string())
    }
}

#[derive(Clone)]
pub enum MergeStrategy {
    Overwrite,
    Recursive,
    CombineArrays,
    Skip,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_overwrite() {
        let mut base = json!({"a": 1, "b": 2});
        let update = json!({"b": 3, "c": 4});
        
        merge_json(&mut base, &update, MergeStrategy::Overwrite).unwrap();
        
        assert_eq!(base["a"], 1);
        assert_eq!(base["b"], 3);
        assert_eq!(base["c"], 4);
    }

    #[test]
    fn test_merge_recursive() {
        let mut base = json!({"a": {"x": 1}, "b": 2});
        let update = json!({"a": {"y": 3}, "c": 4});
        
        merge_json(&mut base, &update, MergeStrategy::Recursive).unwrap();
        
        assert_eq!(base["a"]["x"], 1);
        assert_eq!(base["a"]["y"], 3);
        assert_eq!(base["b"], 2);
        assert_eq!(base["c"], 4);
    }
}