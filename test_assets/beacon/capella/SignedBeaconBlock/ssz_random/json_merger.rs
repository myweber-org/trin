
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

fn merge_objects(base: &mut Map<String, Value>, new: Map<String, Value>) {
    for (key, new_value) in new {
        match base.get_mut(&key) {
            Some(existing_value) => {
                if let (Value::Object(mut existing_obj), Value::Object(new_obj)) = (existing_value.clone(), new_value) {
                    let mut existing_map = if let Value::Object(obj) = existing_obj {
                        obj
                    } else {
                        Map::new()
                    };
                    merge_objects(&mut existing_map, new_obj);
                    base.insert(key, Value::Object(existing_map));
                } else if existing_value != &new_value {
                    let merged_array = Value::Array(vec![existing_value.clone(), new_value]);
                    base.insert(key, merged_array);
                }
            }
            None => {
                base.insert(key, new_value);
            }
        }
    }
}