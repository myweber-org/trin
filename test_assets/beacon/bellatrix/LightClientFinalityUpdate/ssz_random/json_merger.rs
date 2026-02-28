use serde_json::{Map, Value};

pub fn merge_json(base: &mut Value, update: &Value) {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (key, update_value) in update_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    merge_json(base_value, update_value);
                } else {
                    base_map.insert(key.clone(), update_value.clone());
                }
            }
        }
        (base, update) => {
            *base = update.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_json() {
        let mut base = json!({
            "a": 1,
            "b": {
                "c": 2,
                "d": 3
            }
        });

        let update = json!({
            "b": {
                "d": 99,
                "e": 100
            },
            "f": "new"
        });

        merge_json(&mut base, &update);

        assert_eq!(base["a"], 1);
        assert_eq!(base["b"]["c"], 2);
        assert_eq!(base["b"]["d"], 99);
        assert_eq!(base["b"]["e"], 100);
        assert_eq!(base["f"], "new");
    }
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged = Map::new();
    
    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;
        
        if let Value::Object(obj) = json {
            merge_objects(&mut merged, obj);
        }
    }
    
    let output_json = Value::Object(merged);
    let output_str = serde_json::to_string_pretty(&output_json)?;
    fs::write(output_path, output_str)?;
    
    Ok(())
}

fn merge_objects(target: &mut Map<String, Value>, source: Map<String, Value>) {
    for (key, value) in source {
        if !target.contains_key(&key) {
            target.insert(key.clone(), value);
        } else {
            let existing = target.get_mut(&key).unwrap();
            
            match (existing, value) {
                (Value::Object(ref mut target_obj), Value::Object(source_obj)) => {
                    merge_objects(target_obj, source_obj);
                }
                (Value::Array(ref mut target_arr), Value::Array(source_arr)) => {
                    target_arr.extend(source_arr);
                    target_arr.sort();
                    target_arr.dedup();
                }
                _ => {
                    *existing = value;
                }
            }
        }
    }
}