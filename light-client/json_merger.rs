
use serde_json::{Value, Map};
use std::collections::HashSet;

pub enum ConflictResolution {
    PreferFirst,
    PreferSecond,
    MergeArrays,
    FailOnConflict,
}

pub fn merge_json(
    first: &Value,
    second: &Value,
    resolution: ConflictResolution,
) -> Result<Value, String> {
    match (first, second) {
        (Value::Object(first_map), Value::Object(second_map)) => {
            merge_objects(first_map, second_map, resolution)
        }
        (Value::Array(first_arr), Value::Array(second_arr)) => {
            merge_arrays(first_arr, second_arr, resolution)
        }
        _ => handle_primitive_conflict(first, second, resolution),
    }
}

fn merge_objects(
    first: &Map<String, Value>,
    second: &Map<String, Value>,
    resolution: ConflictResolution,
) -> Result<Value, String> {
    let mut result = Map::new();
    let all_keys: HashSet<_> = first.keys().chain(second.keys()).collect();

    for key in all_keys {
        let first_val = first.get(key);
        let second_val = second.get(key);

        match (first_val, second_val) {
            (Some(f), Some(s)) => {
                let merged = merge_json(f, s, resolution.clone())?;
                result.insert(key.clone(), merged);
            }
            (Some(val), None) | (None, Some(val)) => {
                result.insert(key.clone(), val.clone());
            }
            (None, None) => unreachable!(),
        }
    }

    Ok(Value::Object(result))
}

fn merge_arrays(
    first: &[Value],
    second: &[Value],
    resolution: ConflictResolution,
) -> Result<Value, String> {
    match resolution {
        ConflictResolution::MergeArrays => {
            let mut merged = Vec::with_capacity(first.len() + second.len());
            merged.extend_from_slice(first);
            merged.extend_from_slice(second);
            Ok(Value::Array(merged))
        }
        _ => handle_primitive_conflict(
            &Value::Array(first.to_vec()),
            &Value::Array(second.to_vec()),
            resolution,
        ),
    }
}

fn handle_primitive_conflict(
    first: &Value,
    second: &Value,
    resolution: ConflictResolution,
) -> Result<Value, String> {
    if first == second {
        return Ok(first.clone());
    }

    match resolution {
        ConflictResolution::PreferFirst => Ok(first.clone()),
        ConflictResolution::PreferSecond => Ok(second.clone()),
        ConflictResolution::FailOnConflict => Err(format!(
            "Conflict between values: {} and {}",
            first, second
        )),
        ConflictResolution::MergeArrays => Err("Cannot merge non-array values".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_objects_prefer_first() {
        let first = json!({"a": 1, "b": 2});
        let second = json!({"b": 3, "c": 4});
        let result = merge_json(&first, &second, ConflictResolution::PreferFirst).unwrap();
        assert_eq!(result, json!({"a": 1, "b": 2, "c": 4}));
    }

    #[test]
    fn test_merge_arrays() {
        let first = json!([1, 2]);
        let second = json!([3, 4]);
        let result = merge_json(&first, &second, ConflictResolution::MergeArrays).unwrap();
        assert_eq!(result, json!([1, 2, 3, 4]));
    }
}