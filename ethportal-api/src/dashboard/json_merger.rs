use serde_json::{Map, Value};

pub fn merge_json(a: &mut Value, b: &Value) {
    match (a, b) {
        (Value::Object(a_map), Value::Object(b_map)) => {
            for (key, b_value) in b_map {
                if let Some(a_value) = a_map.get_mut(key) {
                    merge_json(a_value, b_value);
                } else {
                    a_map.insert(key.clone(), b_value.clone());
                }
            }
        }
        (a, b) => *a = b.clone(),
    }
}

pub fn merge_json_with_strategy(a: &mut Value, b: &Value, strategy: MergeStrategy) {
    match (a, b) {
        (Value::Object(a_map), Value::Object(b_map)) => {
            for (key, b_value) in b_map {
                if let Some(a_value) = a_map.get_mut(key) {
                    merge_json_with_strategy(a_value, b_value, strategy);
                } else {
                    a_map.insert(key.clone(), b_value.clone());
                }
            }
        }
        (Value::Array(a_arr), Value::Array(b_arr)) => match strategy {
            MergeStrategy::Replace => *a = b.clone(),
            MergeStrategy::Append => a_arr.extend(b_arr.clone()),
            MergeStrategy::Merge => {
                for (i, b_item) in b_arr.iter().enumerate() {
                    if i < a_arr.len() {
                        merge_json_with_strategy(&mut a_arr[i], b_item, strategy);
                    } else {
                        a_arr.push(b_item.clone());
                    }
                }
            }
        },
        (a, b) => *a = b.clone(),
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MergeStrategy {
    Replace,
    Append,
    Merge,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_merge() {
        let mut a = json!({"a": 1, "b": {"c": 2}});
        let b = json!({"b": {"d": 3}, "e": 4});
        
        merge_json(&mut a, &b);
        
        assert_eq!(a, json!({"a": 1, "b": {"c": 2, "d": 3}, "e": 4}));
    }

    #[test]
    fn test_array_append_strategy() {
        let mut a = json!({"items": [1, 2]});
        let b = json!({"items": [3, 4]});
        
        merge_json_with_strategy(&mut a, &b, MergeStrategy::Append);
        
        assert_eq!(a, json!({"items": [1, 2, 3, 4]}));
    }
}