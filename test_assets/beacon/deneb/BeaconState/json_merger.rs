
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if merged.contains_key(&key) {
                    let existing = merged.get(&key).unwrap();
                    if existing != &value {
                        let resolved = resolve_conflict(&key, existing, &value);
                        merged.insert(key, resolved);
                    }
                } else {
                    merged.insert(key, value);
                }
            }
        }
    }

    let output_json = Value::Object(merged);
    let output_str = serde_json::to_string_pretty(&output_json)?;
    fs::write(output_path, output_str)?;
    
    Ok(())
}

fn resolve_conflict(key: &str, existing: &Value, new: &Value) -> Value {
    match (existing, new) {
        (Value::Array(a), Value::Array(b)) => {
            let mut combined = a.clone();
            combined.extend(b.clone());
            Value::Array(combined)
        },
        (Value::Number(_), Value::Number(_)) => new.clone(),
        (Value::String(_), Value::String(_)) => {
            Value::String(format!("{}|{}", existing.as_str().unwrap(), new.as_str().unwrap()))
        },
        _ => new.clone()
    }
}use serde_json::{Value, Map};
use std::collections::HashSet;

pub fn merge_json(base: &mut Value, update: &Value, deep: bool) -> Value {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            if deep {
                let mut result = base_map.clone();
                for (key, update_value) in update_map {
                    if let Some(base_value) = result.get_mut(key) {
                        *base_value = merge_json(base_value, update_value, deep);
                    } else {
                        result.insert(key.clone(), update_value.clone());
                    }
                }
                Value::Object(result)
            } else {
                let mut result = base_map.clone();
                for (key, value) in update_map {
                    result.insert(key.clone(), value.clone());
                }
                Value::Object(result)
            }
        }
        (Value::Array(base_arr), Value::Array(update_arr)) => {
            let mut result = base_arr.clone();
            result.extend_from_slice(update_arr);
            Value::Array(result)
        }
        _ => update.clone(),
    }
}

pub fn merge_json_with_conflict_resolution(
    base: &Value,
    update: &Value,
    conflict_strategy: ConflictStrategy,
) -> Value {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            let mut result = Map::new();
            let base_keys: HashSet<_> = base_map.keys().collect();
            let update_keys: HashSet<_> = update_map.keys().collect();
            
            for key in base_keys.union(&update_keys) {
                let base_val = base_map.get(*key);
                let update_val = update_map.get(*key);
                
                match (base_val, update_val) {
                    (Some(b), Some(u)) => {
                        let merged = merge_json_with_conflict_resolution(b, u, conflict_strategy);
                        result.insert((*key).clone(), merged);
                    }
                    (Some(b), None) => {
                        result.insert((*key).clone(), b.clone());
                    }
                    (None, Some(u)) => {
                        result.insert((*key).clone(), u.clone());
                    }
                    _ => {}
                }
            }
            Value::Object(result)
        }
        (Value::Array(base_arr), Value::Array(update_arr)) => {
            match conflict_strategy {
                ConflictStrategy::PreferBase => Value::Array(base_arr.clone()),
                ConflictStrategy::PreferUpdate => Value::Array(update_arr.clone()),
                ConflictStrategy::Merge => {
                    let mut result = base_arr.clone();
                    result.extend_from_slice(update_arr);
                    Value::Array(result)
                }
            }
        }
        (_, _) => match conflict_strategy {
            ConflictStrategy::PreferBase => base.clone(),
            ConflictStrategy::PreferUpdate => update.clone(),
            ConflictStrategy::Merge => update.clone(),
        },
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ConflictStrategy {
    PreferBase,
    PreferUpdate,
    Merge,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_shallow_merge() {
        let mut base = json!({"a": 1, "b": 2});
        let update = json!({"b": 3, "c": 4});
        
        let result = merge_json(&mut base, &update, false);
        assert_eq!(result, json!({"a": 1, "b": 3, "c": 4}));
    }

    #[test]
    fn test_deep_merge() {
        let mut base = json!({"a": {"x": 1}, "b": 2});
        let update = json!({"a": {"y": 3}, "c": 4});
        
        let result = merge_json(&mut base, &update, true);
        assert_eq!(result, json!({"a": {"x": 1, "y": 3}, "b": 2, "c": 4}));
    }
}