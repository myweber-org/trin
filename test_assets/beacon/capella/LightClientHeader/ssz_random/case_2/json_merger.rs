
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_data = JsonValue::Object(serde_json::Map::new());

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", file_path);
            continue;
        }

        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let json_data: JsonValue = serde_json::from_str(&contents)?;

        if let JsonValue::Object(map) = json_data {
            if let JsonValue::Object(merged_map) = &mut merged_data {
                for (key, value) in map {
                    merged_map.insert(key, value);
                }
            }
        } else {
            eprintln!("Warning: File {} does not contain a JSON object, skipping.", file_path);
        }
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &merged_data)?;

    Ok(())
}

pub fn merge_json_with_strategy(
    file_paths: &[&str],
    output_path: &str,
    strategy: MergeStrategy,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_data: HashMap<String, JsonValue> = HashMap::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", file_path);
            continue;
        }

        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let json_data: JsonValue = serde_json::from_str(&contents)?;

        if let JsonValue::Object(map) = json_data {
            for (key, value) in map {
                match strategy {
                    MergeStrategy::Overwrite => {
                        merged_data.insert(key, value);
                    }
                    MergeStrategy::SkipIfExists => {
                        merged_data.entry(key).or_insert(value);
                    }
                    MergeStrategy::MergeArrays => {
                        if let Some(existing) = merged_data.get_mut(&key) {
                            if existing.is_array() && value.is_array() {
                                if let JsonValue::Array(existing_arr) = existing {
                                    if let JsonValue::Array(new_arr) = value {
                                        existing_arr.extend(new_arr);
                                    }
                                }
                            } else {
                                merged_data.insert(key, value);
                            }
                        } else {
                            merged_data.insert(key, value);
                        }
                    }
                }
            }
        } else {
            eprintln!("Warning: File {} does not contain a JSON object, skipping.", file_path);
        }
    }

    let json_output = JsonValue::Object(
        merged_data
            .into_iter()
            .map(|(k, v)| (k, v))
            .collect::<serde_json::Map<_, _>>(),
    );

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &json_output)?;

    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub enum MergeStrategy {
    Overwrite,
    SkipIfExists,
    MergeArrays,
}