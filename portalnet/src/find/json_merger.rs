use serde_json::{Map, Value};
use std::collections::HashSet;

pub fn merge_json(base: &mut Value, update: &Value, deep: bool) -> Result<(), String> {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (key, update_value) in update_map {
                if deep {
                    if let Some(base_value) = base_map.get_mut(key) {
                        merge_json(base_value, update_value, deep)?;
                    } else {
                        base_map.insert(key.clone(), update_value.clone());
                    }
                } else {
                    base_map.insert(key.clone(), update_value.clone());
                }
            }
            Ok(())
        }
        (Value::Array(base_arr), Value::Array(update_arr)) => {
            if deep {
                base_arr.extend(update_arr.clone());
                let unique: HashSet<_> = base_arr.drain(..).collect();
                base_arr.extend(unique);
                base_arr.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
            } else {
                *base_arr = update_arr.clone();
            }
            Ok(())
        }
        (base_val, update_val) => {
            if deep && base_val.is_null() {
                *base_val = update_val.clone();
                Ok(())
            } else {
                *base_val = update_val.clone();
                Ok(())
            }
        }
    }
}

pub fn merge_json_with_strategy(
    base: &mut Value,
    update: &Value,
    strategy: MergeStrategy,
) -> Result<(), String> {
    match strategy {
        MergeStrategy::Shallow => merge_json(base, update, false),
        MergeStrategy::Deep => merge_json(base, update, true),
        MergeStrategy::Custom(merger) => merger(base, update),
    }
}

pub enum MergeStrategy {
    Shallow,
    Deep,
    Custom(fn(&mut Value, &Value) -> Result<(), String>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_shallow_merge() {
        let mut base = json!({"a": 1, "b": {"inner": 2}});
        let update = json!({"b": {"new": 3}, "c": 4});
        merge_json(&mut base, &update, false).unwrap();
        assert_eq!(base["b"], json!({"new": 3}));
    }

    #[test]
    fn test_deep_merge() {
        let mut base = json!({"a": 1, "b": {"inner": 2}});
        let update = json!({"b": {"new": 3}, "c": 4});
        merge_json(&mut base, &update, true).unwrap();
        assert_eq!(base["b"], json!({"inner": 2, "new": 3}));
    }
}