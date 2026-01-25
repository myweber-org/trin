
use serde_json::{Map, Value};

pub fn merge_json(base: &mut Value, update: &Value, resolve_conflicts: bool) -> Result<(), String> {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (key, update_value) in update_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    if base_value == update_value {
                        continue;
                    }
                    
                    if resolve_conflicts {
                        if let (Value::Object(_), Value::Object(_)) = (base_value, update_value) {
                            merge_json(base_value, update_value, resolve_conflicts)?;
                        } else {
                            *base_value = update_value.clone();
                        }
                    } else {
                        return Err(format!("Conflict detected for key '{}'", key));
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

pub fn merge_json_with_strategy(
    base: &mut Value,
    update: &Value,
    strategy: MergeStrategy,
) -> Result<(), String> {
    match strategy {
        MergeStrategy::PreferBase => Ok(()),
        MergeStrategy::PreferUpdate => {
            *base = update.clone();
            Ok(())
        }
        MergeStrategy::Recursive => merge_json(base, update, true),
        MergeStrategy::Strict => merge_json(base, update, false),
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MergeStrategy {
    PreferBase,
    PreferUpdate,
    Recursive,
    Strict,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_without_conflicts() {
        let mut base = json!({"a": 1, "b": 2});
        let update = json!({"c": 3, "d": 4});
        
        merge_json(&mut base, &update, true).unwrap();
        assert_eq!(base, json!({"a": 1, "b": 2, "c": 3, "d": 4}));
    }

    #[test]
    fn test_merge_with_conflict_strict() {
        let mut base = json!({"a": 1});
        let update = json!({"a": 2});
        
        let result = merge_json(&mut base, &update, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_recursive_merge() {
        let mut base = json!({
            "a": {"x": 1},
            "b": 2
        });
        let update = json!({
            "a": {"y": 3},
            "c": 4
        });
        
        merge_json(&mut base, &update, true).unwrap();
        assert_eq!(base, json!({
            "a": {"x": 1, "y": 3},
            "b": 2,
            "c": 4
        }));
    }
}