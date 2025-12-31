
use std::collections::HashMap;
use serde_json::{Value, Map};

pub fn merge_json(base: &mut Value, update: &Value, resolve_conflict: fn(&Value, &Value) -> Value) {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (key, update_value) in update_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    merge_json(base_value, update_value, resolve_conflict);
                } else {
                    base_map.insert(key.clone(), update_value.clone());
                }
            }
        }
        (Value::Array(base_arr), Value::Array(update_arr)) => {
            base_arr.extend(update_arr.clone());
        }
        (base_val, update_val) => {
            if base_val != update_val {
                *base_val = resolve_conflict(base_val, update_val);
            }
        }
    }
}

pub fn merge_json_with_default_strategy(base: &mut Value, update: &Value) {
    let default_resolver = |_old: &Value, new: &Value| new.clone();
    merge_json(base, update, default_resolver);
}

pub fn merge_multiple_json(documents: Vec<Value>, resolver: fn(&Value, &Value) -> Value) -> Option<Value> {
    if documents.is_empty() {
        return None;
    }

    let mut result = documents[0].clone();
    for doc in documents.iter().skip(1) {
        merge_json(&mut result, doc, resolver);
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_merge() {
        let mut base = json!({"a": 1, "b": {"c": 2}});
        let update = json!({"b": {"d": 3}, "e": 4});
        merge_json_with_default_strategy(&mut base, &update);
        assert_eq!(base, json!({"a": 1, "b": {"c": 2, "d": 3}, "e": 4}));
    }

    #[test]
    fn test_conflict_resolution() {
        let custom_resolver = |old: &Value, new: &Value| {
            if let (Some(old_num), Some(new_num)) = (old.as_i64(), new.as_i64()) {
                json!(old_num + new_num)
            } else {
                new.clone()
            }
        };

        let mut base = json!({"count": 5});
        let update = json!({"count": 3});
        merge_json(&mut base, &update, custom_resolver);
        assert_eq!(base, json!({"count": 8}));
    }

    #[test]
    fn test_array_merge() {
        let mut base = json!([1, 2]);
        let update = json!([3, 4]);
        merge_json_with_default_strategy(&mut base, &update);
        assert_eq!(base, json!([1, 2, 3, 4]));
    }
}
use serde_json::{Map, Value};
use std::env;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <output_file.json> <input1.json> [input2.json ...]", args[0]);
        std::process::exit(1);
    }

    let output_path = &args[1];
    let input_paths = &args[2..];

    let mut merged_map = Map::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        if !path.exists() {
            eprintln!("Warning: File '{}' not found, skipping.", input_path);
            continue;
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Key '{}' already exists, overwriting.", key);
                }
                merged_map.insert(key, value);
            }
        } else {
            eprintln!("Warning: '{}' does not contain a JSON object, skipping.", input_path);
        }
    }

    let output_file = File::create(output_path)?;
    let formatted_json = serde_json::to_string_pretty(&Value::Object(merged_map))?;
    write!(&output_file, "{}", formatted_json)?;

    println!("Successfully merged {} file(s) into '{}'", input_paths.len(), output_path);
    Ok(())
}