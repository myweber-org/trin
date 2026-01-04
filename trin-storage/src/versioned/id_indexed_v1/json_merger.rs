
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
    let serialized = serde_json::to_string_pretty(&output_json)?;
    fs::write(output_path, serialized)?;
    
    Ok(())
}

fn merge_objects(base: &mut Map<String, Value>, new: Map<String, Value>) {
    for (key, new_value) in new {
        if let Some(existing_value) = base.get_mut(&key) {
            match (existing_value, new_value) {
                (Value::Object(ref mut base_obj), Value::Object(new_obj)) => {
                    merge_objects(base_obj, new_obj);
                }
                (Value::Array(ref mut base_arr), Value::Array(new_arr)) => {
                    base_arr.extend(new_arr);
                    base_arr.sort();
                    base_arr.dedup();
                }
                _ => {
                    *existing_value = new_value;
                }
            }
        } else {
            base.insert(key, new_value);
        }
    }
}