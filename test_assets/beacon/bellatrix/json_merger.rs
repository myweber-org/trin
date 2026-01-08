use std::collections::HashMap;
use serde_json::{Value, Map};

pub fn merge_json(base: &mut Value, extension: &Value) -> Value {
    match (base, extension) {
        (Value::Object(base_map), Value::Object(ext_map)) => {
            for (key, ext_value) in ext_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    *base_value = merge_json(base_value, ext_value);
                } else {
                    base_map.insert(key.clone(), ext_value.clone());
                }
            }
            Value::Object(base_map.clone())
        }
        (_, ext_value) => ext_value.clone(),
    }
}

pub fn merge_json_with_strategy(
    base: &Value,
    extension: &Value,
    array_merge_strategy: ArrayMergeStrategy,
) -> Value {
    match (base, extension) {
        (Value::Object(base_map), Value::Object(ext_map)) => {
            let mut result = Map::new();
            
            let all_keys: Vec<_> = base_map.keys().chain(ext_map.keys()).collect();
            let unique_keys: std::collections::HashSet<_> = all_keys.into_iter().collect();
            
            for key in unique_keys {
                match (base_map.get(key), ext_map.get(key)) {
                    (Some(base_val), Some(ext_val)) => {
                        let merged = merge_json_with_strategy(base_val, ext_val, array_merge_strategy);
                        result.insert(key.clone(), merged);
                    }
                    (Some(val), None) | (None, Some(val)) => {
                        result.insert(key.clone(), val.clone());
                    }
                    (None, None) => unreachable!(),
                }
            }
            
            Value::Object(result)
        }
        (Value::Array(base_arr), Value::Array(ext_arr)) => {
            match array_merge_strategy {
                ArrayMergeStrategy::Concat => {
                    let mut combined = base_arr.clone();
                    combined.extend(ext_arr.clone());
                    Value::Array(combined)
                }
                ArrayMergeStrategy::Replace => Value::Array(ext_arr.clone()),
                ArrayMergeStrategy::MergeUnique => {
                    let mut seen = HashMap::new();
                    let mut result = Vec::new();
                    
                    for item in base_arr.iter().chain(ext_arr.iter()) {
                        let key = format!("{:?}", item);
                        if !seen.contains_key(&key) {
                            seen.insert(key, true);
                            result.push(item.clone());
                        }
                    }
                    
                    Value::Array(result)
                }
            }
        }
        (_, ext_value) => ext_value.clone(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArrayMergeStrategy {
    Concat,
    Replace,
    MergeUnique,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_merge() {
        let mut base = json!({"a": 1, "b": {"c": 2}});
        let extension = json!({"b": {"d": 3}, "e": 4});
        
        let result = merge_json(&mut base, &extension);
        
        assert_eq!(result["a"], 1);
        assert_eq!(result["b"]["c"], 2);
        assert_eq!(result["b"]["d"], 3);
        assert_eq!(result["e"], 4);
    }

    #[test]
    fn test_array_concat() {
        let base = json!({"items": [1, 2]});
        let extension = json!({"items": [3, 4]});
        
        let result = merge_json_with_strategy(&base, &extension, ArrayMergeStrategy::Concat);
        
        assert_eq!(result["items"], json!([1, 2, 3, 4]));
    }

    #[test]
    fn test_array_merge_unique() {
        let base = json!({"data": [1, 2, 3]});
        let extension = json!({"data": [3, 4, 5]});
        
        let result = merge_json_with_strategy(&base, &extension, ArrayMergeStrategy::MergeUnique);
        
        let result_array = result["data"].as_array().unwrap();
        assert!(result_array.contains(&json!(1)));
        assert!(result_array.contains(&json!(2)));
        assert!(result_array.contains(&json!(3)));
        assert!(result_array.contains(&json!(4)));
        assert!(result_array.contains(&json!(5)));
        assert_eq!(result_array.len(), 5);
    }
}