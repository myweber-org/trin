
use serde_json::{Map, Value};
use std::collections::HashSet;

pub enum ConflictResolution {
    PreferFirst,
    PreferSecond,
    MergeArrays,
    FailOnConflict,
}

pub fn merge_json_objects(
    first: &Map<String, Value>,
    second: &Map<String, Value>,
    resolution: ConflictResolution,
) -> Result<Map<String, Value>, String> {
    let mut result = Map::new();
    let mut all_keys: HashSet<&String> = first.keys().chain(second.keys()).collect();

    for key in all_keys {
        let first_val = first.get(key);
        let second_val = second.get(key);

        match (first_val, second_val) {
            (Some(f), None) => {
                result.insert(key.clone(), f.clone());
            }
            (None, Some(s)) => {
                result.insert(key.clone(), s.clone());
            }
            (Some(f), Some(s)) => {
                let merged_value = merge_values(f, s, &resolution)?;
                result.insert(key.clone(), merged_value);
            }
            _ => unreachable!(),
        }
    }

    Ok(result)
}

fn merge_values(
    first: &Value,
    second: &Value,
    resolution: &ConflictResolution,
) -> Result<Value, String> {
    match (first, second) {
        (Value::Object(f_obj), Value::Object(s_obj)) => {
            let merged_map = merge_json_objects(f_obj, s_obj, resolution.clone())?;
            Ok(Value::Object(merged_map))
        }
        (Value::Array(f_arr), Value::Array(s_arr)) => match resolution {
            ConflictResolution::MergeArrays => {
                let mut merged = f_arr.clone();
                merged.extend(s_arr.clone());
                Ok(Value::Array(merged))
            }
            _ => resolve_conflict(first, second, resolution),
        },
        _ => resolve_conflict(first, second, resolution),
    }
}

fn resolve_conflict(first: &Value, second: &Value, resolution: &ConflictResolution) -> Result<Value, String> {
    match resolution {
        ConflictResolution::PreferFirst => Ok(first.clone()),
        ConflictResolution::PreferSecond => Ok(second.clone()),
        ConflictResolution::FailOnConflict => Err(format!(
            "Conflict detected between values: {:?} and {:?}",
            first, second
        )),
        ConflictResolution::MergeArrays => Err("Cannot merge non-array values with MergeArrays strategy".to_string()),
    }
}