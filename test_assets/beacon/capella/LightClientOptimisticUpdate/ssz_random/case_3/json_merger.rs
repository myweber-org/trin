use serde_json::{Value, Map};
use std::collections::HashSet;

pub fn merge_json(base: &mut Value, extension: &Value, deep: bool) {
    match (base, extension) {
        (Value::Object(base_map), Value::Object(ext_map)) => {
            for (key, ext_value) in ext_map {
                if deep {
                    if let Some(base_value) = base_map.get_mut(key) {
                        merge_json(base_value, ext_value, deep);
                    } else {
                        base_map.insert(key.clone(), ext_value.clone());
                    }
                } else {
                    base_map.insert(key.clone(), ext_value.clone());
                }
            }
        }
        (Value::Array(base_arr), Value::Array(ext_arr)) => {
            base_arr.extend(ext_arr.clone());
        }
        (base_val, ext_val) => {
            *base_val = ext_val.clone();
        }
    }
}

pub fn merge_json_with_conflict_resolution(
    base: &mut Value,
    extension: &Value,
    conflict_strategy: ConflictStrategy,
) {
    let mut visited_keys = HashSet::new();
    merge_json_internal(base, extension, &conflict_strategy, &mut visited_keys);
}

fn merge_json_internal(
    base: &mut Value,
    extension: &Value,
    strategy: &ConflictStrategy,
    visited: &mut HashSet<String>,
) {
    match (base, extension) {
        (Value::Object(base_map), Value::Object(ext_map)) => {
            for (key, ext_value) in ext_map {
                let path_key = format!("{}.{}", visited.iter().last().unwrap_or(&String::new()), key);
                visited.insert(path_key.clone());
                
                if let Some(base_value) = base_map.get_mut(key) {
                    match strategy {
                        ConflictStrategy::PreferBase => continue,
                        ConflictStrategy::PreferExtension => {
                            *base_value = ext_value.clone();
                        }
                        ConflictStrategy::MergeDeep => {
                            merge_json_internal(base_value, ext_value, strategy, visited);
                        }
                        ConflictStrategy::CombineArrays => {
                            if let (Value::Array(base_arr), Value::Array(ext_arr)) = (base_value, ext_value) {
                                let mut combined = base_arr.clone();
                                combined.extend(ext_arr.clone());
                                *base_value = Value::Array(combined);
                            } else {
                                merge_json_internal(base_value, ext_value, strategy, visited);
                            }
                        }
                    }
                } else {
                    base_map.insert(key.clone(), ext_value.clone());
                }
                visited.remove(&path_key);
            }
        }
        (Value::Array(base_arr), Value::Array(ext_arr)) => {
            base_arr.extend(ext_arr.clone());
        }
        (base_val, ext_val) => {
            *base_val = ext_val.clone();
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ConflictStrategy {
    PreferBase,
    PreferExtension,
    MergeDeep,
    CombineArrays,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_shallow_merge() {
        let mut base = json!({"a": 1, "b": {"inner": 2}});
        let extension = json!({"b": 3, "c": 4});
        
        merge_json(&mut base, &extension, false);
        
        assert_eq!(base["b"], 3);
        assert_eq!(base["c"], 4);
    }

    #[test]
    fn test_deep_merge() {
        let mut base = json!({"a": 1, "b": {"inner": 2, "keep": 5}});
        let extension = json!({"b": {"inner": 3, "new": 4}});
        
        merge_json(&mut base, &extension, true);
        
        assert_eq!(base["b"]["inner"], 3);
        assert_eq!(base["b"]["keep"], 5);
        assert_eq!(base["b"]["new"], 4);
    }
}