
use std::collections::HashMap;
use serde_json::{Value, Map};

pub fn merge_json(base: &mut Value, update: &Value, resolve_conflict: fn(&Value, &Value) -> Value) -> Value {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (key, update_val) in update_map {
                if let Some(base_val) = base_map.get_mut(key) {
                    let merged = merge_json(base_val, update_val, resolve_conflict);
                    base_map.insert(key.clone(), merged);
                } else {
                    base_map.insert(key.clone(), update_val.clone());
                }
            }
            Value::Object(std::mem::take(base_map))
        }
        (base_val, update_val) if base_val != update_val => {
            resolve_conflict(base_val, update_val)
        }
        _ => base.clone(),
    }
}

pub fn default_resolver(left: &Value, right: &Value) -> Value {
    match (left, right) {
        (Value::Array(left_arr), Value::Array(right_arr)) => {
            let mut merged = left_arr.clone();
            merged.extend(right_arr.clone());
            Value::Array(merged)
        }
        _ => right.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_objects() {
        let mut base = json!({"a": 1, "b": {"x": 10}});
        let update = json!({"b": {"y": 20}, "c": 3});
        let result = merge_json(&mut base, &update, default_resolver);
        
        assert_eq!(result["a"], 1);
        assert_eq!(result["b"]["x"], 10);
        assert_eq!(result["b"]["y"], 20);
        assert_eq!(result["c"], 3);
    }

    #[test]
    fn test_resolve_array_conflict() {
        let mut base = json!({"items": [1, 2]});
        let update = json!({"items": [3, 4]});
        let result = merge_json(&mut base, &update, default_resolver);
        
        assert_eq!(result["items"], json!([1, 2, 3, 4]));
    }
}