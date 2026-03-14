
use serde_json::{Map, Value};
use std::collections::HashSet;

pub enum ConflictResolution {
    PreferFirst,
    PreferSecond,
    MergeArrays,
    FailOnConflict,
}

pub fn merge_json(
    first: &Map<String, Value>,
    second: &Map<String, Value>,
    resolution: ConflictResolution,
) -> Result<Map<String, Value>, String> {
    let mut result = Map::new();
    let mut all_keys: HashSet<String> = first.keys().chain(second.keys()).cloned().collect();

    for key in all_keys {
        let first_val = first.get(&key);
        let second_val = second.get(&key);

        match (first_val, second_val) {
            (Some(f), Some(s)) => {
                if f == s {
                    result.insert(key.clone(), f.clone());
                } else {
                    match resolution {
                        ConflictResolution::PreferFirst => {
                            result.insert(key.clone(), f.clone());
                        }
                        ConflictResolution::PreferSecond => {
                            result.insert(key.clone(), s.clone());
                        }
                        ConflictResolution::MergeArrays => {
                            if f.is_array() && s.is_array() {
                                let mut merged_array = Vec::new();
                                if let Value::Array(arr1) = f {
                                    merged_array.extend(arr1.clone());
                                }
                                if let Value::Array(arr2) = s {
                                    merged_array.extend(arr2.clone());
                                }
                                result.insert(key.clone(), Value::Array(merged_array));
                            } else {
                                return Err(format!(
                                    "Conflict on key '{}': both values are not arrays",
                                    key
                                ));
                            }
                        }
                        ConflictResolution::FailOnConflict => {
                            return Err(format!("Conflict on key '{}'", key));
                        }
                    }
                }
            }
            (Some(val), None) => {
                result.insert(key.clone(), val.clone());
            }
            (None, Some(val)) => {
                result.insert(key.clone(), val.clone());
            }
            (None, None) => unreachable!(),
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_prefer_first() {
        let mut first = Map::new();
        first.insert("a".to_string(), json!(1));
        first.insert("b".to_string(), json!("test"));

        let mut second = Map::new();
        second.insert("a".to_string(), json!(2));
        second.insert("c".to_string(), json!(true));

        let merged = merge_json(&first, &second, ConflictResolution::PreferFirst).unwrap();
        assert_eq!(merged.get("a"), Some(&json!(1)));
        assert_eq!(merged.get("b"), Some(&json!("test")));
        assert_eq!(merged.get("c"), Some(&json!(true)));
    }

    #[test]
    fn test_merge_arrays() {
        let mut first = Map::new();
        first.insert("items".to_string(), json!([1, 2]));

        let mut second = Map::new();
        second.insert("items".to_string(), json!([3, 4]));

        let merged = merge_json(&first, &second, ConflictResolution::MergeArrays).unwrap();
        if let Value::Array(arr) = merged.get("items").unwrap() {
            assert_eq!(arr, &vec![json!(1), json!(2), json!(3), json!(4)]);
        } else {
            panic!("Expected array");
        }
    }
}