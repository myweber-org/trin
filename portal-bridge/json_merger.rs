
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::Path;

use serde_json::{Value, Map};

pub fn merge_json_files(input_paths: &[String], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map: Map<String, Value> = Map::new();

    for path_str in input_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            eprintln!("Warning: File {} does not exist, skipping.", path_str);
            continue;
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Key '{}' already exists, overwriting with value from {}.", key, path_str);
                }
                merged_map.insert(key, value);
            }
        } else {
            eprintln!("Warning: File {} does not contain a JSON object, skipping.", path_str);
        }
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &Value::Object(merged_map))?;

    println!("Successfully merged {} JSON files into {}.", input_paths.len(), output_path);
    Ok(())
}

pub fn merge_json_files_with_strategy(
    input_paths: &[String],
    output_path: &str,
    strategy: MergeStrategy,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut accumulator: HashMap<String, Value> = HashMap::new();

    for path_str in input_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            eprintln!("Warning: File {} does not exist, skipping.", path_str);
            continue;
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                match strategy {
                    MergeStrategy::Overwrite => {
                        accumulator.insert(key, value);
                    }
                    MergeStrategy::MergeObjects => {
                        if let Some(existing) = accumulator.get_mut(&key) {
                            if let (Value::Object(existing_obj), Value::Object(new_obj)) = (existing, &value) {
                                let mut combined = existing_obj.clone();
                                for (k, v) in new_obj {
                                    combined.insert(k.clone(), v.clone());
                                }
                                *existing = Value::Object(combined);
                            } else {
                                accumulator.insert(key, value);
                            }
                        } else {
                            accumulator.insert(key, value);
                        }
                    }
                    MergeStrategy::SkipDuplicate => {
                        accumulator.entry(key).or_insert(value);
                    }
                }
            }
        } else {
            eprintln!("Warning: File {} does not contain a JSON object, skipping.", path_str);
        }
    }

    let output_map: Map<String, Value> = accumulator.into_iter().collect();
    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &Value::Object(output_map))?;

    println!("Successfully merged {} JSON files into {} using {:?} strategy.", input_paths.len(), output_path, strategy);
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub enum MergeStrategy {
    Overwrite,
    MergeObjects,
    SkipDuplicate,
}use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(Value::Object(merged_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"name": "Alice", "age": 30}"#).unwrap();
        writeln!(file2, r#"{"city": "Berlin", "active": true}"#).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("name").unwrap(), "Alice");
        assert_eq!(obj.get("age").unwrap(), 30);
        assert_eq!(obj.get("city").unwrap(), "Berlin");
        assert_eq!(obj.get("active").unwrap(), true);
    }
}
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), String> {
    if paths.is_empty() {
        return Err("No input files provided".to_string());
    }

    let mut merged = Map::new();
    let mut conflict_log = Vec::new();

    for (idx, path) in paths.iter().enumerate() {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        
        let json: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;
        
        if let Value::Object(obj) = json {
            merge_object(&mut merged, obj, idx, &mut conflict_log);
        } else {
            return Err(format!("Expected JSON object in {}", path.as_ref().display()));
        }
    }

    if !conflict_log.is_empty() {
        let log_path = output_path.as_ref().with_extension("conflicts.log");
        fs::write(&log_path, conflict_log.join("\n"))
            .map_err(|e| format!("Failed to write conflict log: {}", e))?;
        println!("Conflicts detected, see {}", log_path.display());
    }

    let output_json = Value::Object(merged);
    let pretty_json = serde_json::to_string_pretty(&output_json)
        .map_err(|e| format!("Failed to serialize merged JSON: {}", e))?;
    
    fs::write(&output_path, pretty_json)
        .map_err(|e| format!("Failed to write output file: {}", e))?;
    
    Ok(())
}

fn merge_object(base: &mut Map<String, Value>, 
                new: Map<String, Value>, 
                file_index: usize,
                conflicts: &mut Vec<String>) {
    for (key, new_value) in new {
        match base.get_mut(&key) {
            Some(existing_value) => {
                handle_conflict(key, existing_value, new_value, file_index, conflicts);
            }
            None => {
                base.insert(key, new_value);
            }
        }
    }
}

fn handle_conflict(key: String, 
                   existing: &mut Value, 
                   new: Value, 
                   file_index: usize,
                   conflicts: &mut Vec<String>) {
    match (existing, new) {
        (Value::Object(ref mut existing_obj), Value::Object(new_obj)) => {
            merge_object(existing_obj, new_obj, file_index, conflicts);
        }
        (Value::Array(ref mut existing_arr), Value::Array(new_arr)) => {
            merge_array(existing_arr, new_arr, file_index, &key, conflicts);
        }
        (existing_val, new_val) if existing_val == &new_val => {
            // Values are identical, no conflict
        }
        _ => {
            conflicts.push(format!(
                "Conflict at key '{}': existing value {:?} conflicts with new value {:?} from file {}",
                key, existing, new, file_index
            ));
        }
    }
}

fn merge_array(existing: &mut Vec<Value>, 
               new: Vec<Value>, 
               file_index: usize,
               key: &str,
               conflicts: &mut Vec<String>) {
    let existing_set: HashSet<_> = existing.iter().collect();
    let new_set: HashSet<_> = new.iter().collect();
    
    let unique_new: Vec<_> = new.into_iter()
        .filter(|v| !existing_set.contains(v))
        .collect();
    
    if !unique_new.is_empty() {
        existing.extend(unique_new);
    } else if new_set != existing_set {
        conflicts.push(format!(
            "Array conflict at key '{}': duplicate entries with different ordering in file {}",
            key, file_index
        ));
    }
}