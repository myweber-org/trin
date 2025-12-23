
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), String> {
    if paths.is_empty() {
        return Err("No input files provided".to_string());
    }

    let mut merged_map = Map::new();
    let mut conflict_log = Vec::new();

    for (idx, path) in paths.iter().enumerate() {
        let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path.as_ref().display(), e))?;
        let reader = BufReader::new(file);
        let json: Map<String, Value> = serde_json::from_reader(reader)
            .map_err(|e| format!("Failed to parse {}: {}", path.as_ref().display(), e))?;

        for (key, value) in json {
            if let Some(existing) = merged_map.get(&key) {
                if existing != &value {
                    conflict_log.push(format!(
                        "Conflict at key '{}': file {} has {:?}, previous had {:?}",
                        key,
                        idx + 1,
                        value,
                        existing
                    ));
                    merged_map.insert(format!("{}_conflict_{}", key, idx + 1), value);
                }
            } else {
                merged_map.insert(key, value);
            }
        }
    }

    let mut output_map = Map::new();
    output_map.insert("data".to_string(), Value::Object(merged_map));
    
    if !conflict_log.is_empty() {
        output_map.insert("conflicts".to_string(), Value::Array(
            conflict_log.into_iter().map(Value::String).collect()
        ));
    }

    let output_file = File::create(&output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    serde_json::to_writer_pretty(output_file, &Value::Object(output_map))
        .map_err(|e| format!("Failed to write JSON: {}", e))?;

    Ok(())
}

pub fn find_unique_keys<P: AsRef<Path>>(paths: &[P]) -> Result<Vec<HashSet<String>>, String> {
    let mut key_sets = Vec::new();
    
    for path in paths {
        let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path.as_ref().display(), e))?;
        let reader = BufReader::new(file);
        let json: Map<String, Value> = serde_json::from_reader(reader)
            .map_err(|e| format!("Failed to parse {}: {}", path.as_ref().display(), e))?;
        
        key_sets.push(json.keys().cloned().collect());
    }
    
    Ok(key_sets)
}