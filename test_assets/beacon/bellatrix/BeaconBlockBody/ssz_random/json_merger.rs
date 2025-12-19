
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                merge_value(&mut merged, key, value);
            }
        }
    }

    Ok(Value::Object(merged))
}

fn merge_value(map: &mut Map<String, Value>, key: String, new_value: Value) {
    match map.get_mut(&key) {
        Some(existing) => {
            if let (Value::Object(existing_obj), Value::Object(new_obj)) = (existing, &new_value) {
                let mut existing_obj = existing_obj.clone();
                for (k, v) in new_obj {
                    merge_value(&mut existing_obj, k.clone(), v.clone());
                }
                map.insert(key, Value::Object(existing_obj));
            } else if let (Value::Array(existing_arr), Value::Array(new_arr)) = (existing, &new_value) {
                let mut combined = existing_arr.clone();
                combined.extend(new_arr.clone());
                map.insert(key, Value::Array(combined));
            } else {
                map.insert(key, new_value);
            }
        }
        None => {
            map.insert(key, new_value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_merge() {
        let data1 = json!({"name": "Alice", "age": 30});
        let data2 = json!({"city": "Berlin", "age": 31});

        let merged = merge_json(&data1, &data2);
        assert_eq!(merged["name"], "Alice");
        assert_eq!(merged["age"], 31);
        assert_eq!(merged["city"], "Berlin");
    }

    fn merge_json(a: &Value, b: &Value) -> Value {
        let mut map = Map::new();
        
        if let Value::Object(obj) = a {
            for (k, v) in obj {
                map.insert(k.clone(), v.clone());
            }
        }
        
        if let Value::Object(obj) = b {
            for (k, v) in obj {
                merge_value(&mut map, k.clone(), v.clone());
            }
        }
        
        Value::Object(map)
    }
}