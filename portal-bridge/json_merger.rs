
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
}