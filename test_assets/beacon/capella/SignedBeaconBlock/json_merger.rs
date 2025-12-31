
use std::collections::HashMap;
use serde_json::{Value, Map};

pub fn merge_json(base: &mut Value, update: &Value, resolve_conflict: fn(&Value, &Value) -> Value) {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (key, update_value) in update_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    merge_json(base_value, update_value, resolve_conflict);
                } else {
                    base_map.insert(key.clone(), update_value.clone());
                }
            }
        }
        (Value::Array(base_arr), Value::Array(update_arr)) => {
            base_arr.extend(update_arr.clone());
        }
        (base_val, update_val) => {
            if base_val != update_val {
                *base_val = resolve_conflict(base_val, update_val);
            }
        }
    }
}

pub fn merge_json_with_default_strategy(base: &mut Value, update: &Value) {
    let default_resolver = |_old: &Value, new: &Value| new.clone();
    merge_json(base, update, default_resolver);
}

pub fn merge_multiple_json(documents: Vec<Value>, resolver: fn(&Value, &Value) -> Value) -> Option<Value> {
    if documents.is_empty() {
        return None;
    }

    let mut result = documents[0].clone();
    for doc in documents.iter().skip(1) {
        merge_json(&mut result, doc, resolver);
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_merge() {
        let mut base = json!({"a": 1, "b": {"c": 2}});
        let update = json!({"b": {"d": 3}, "e": 4});
        merge_json_with_default_strategy(&mut base, &update);
        assert_eq!(base, json!({"a": 1, "b": {"c": 2, "d": 3}, "e": 4}));
    }

    #[test]
    fn test_conflict_resolution() {
        let custom_resolver = |old: &Value, new: &Value| {
            if let (Some(old_num), Some(new_num)) = (old.as_i64(), new.as_i64()) {
                json!(old_num + new_num)
            } else {
                new.clone()
            }
        };

        let mut base = json!({"count": 5});
        let update = json!({"count": 3});
        merge_json(&mut base, &update, custom_resolver);
        assert_eq!(base, json!({"count": 8}));
    }

    #[test]
    fn test_array_merge() {
        let mut base = json!([1, 2]);
        let update = json!([3, 4]);
        merge_json_with_default_strategy(&mut base, &update);
        assert_eq!(base, json!([1, 2, 3, 4]));
    }
}