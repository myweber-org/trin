
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(input_paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_object = Map::new();

    for input_path in input_paths {
        let content = fs::read_to_string(input_path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                merged_object.insert(key, value);
            }
        } else {
            return Err("Input JSON is not an object".into());
        }
    }

    let merged_json = Value::Object(merged_object);
    let json_string = serde_json::to_string_pretty(&merged_json)?;
    fs::write(output_path, json_string)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": 2}"#).unwrap();
        fs::write(&file2, r#"{"c": 3, "d": 4}"#).unwrap();

        let inputs = [file1.path(), file2.path()];
        merge_json_files(&inputs, output_file.path()).unwrap();

        let content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        assert_eq!(parsed["a"], 1);
        assert_eq!(parsed["b"], 2);
        assert_eq!(parsed["c"], 3);
        assert_eq!(parsed["d"], 4);
    }
}use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct Record {
    id: u64,
    name: String,
    value: f64,
}

fn load_json_records<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let records: Vec<Record> = serde_json::from_reader(reader)?;
    Ok(records)
}

fn merge_json_files(
    input_paths: &[String],
    output_path: &str,
    key_field: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();
    let mut seen_keys = HashSet::new();

    for path in input_paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_array: Vec<Value> = serde_json::from_reader(reader)?;

        for item in json_array {
            if let Some(obj) = item.as_object() {
                if let Some(key_value) = obj.get(key_field) {
                    let key_string = key_value.to_string();
                    if !seen_keys.contains(&key_string) {
                        seen_keys.insert(key_string.clone());
                        merged_map.insert(key_string, Value::Object(obj.clone()));
                    }
                }
            }
        }
    }

    let output_array: Vec<Value> = merged_map.values().cloned().collect();
    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    serde_json::to_writer_pretty(writer, &output_array)?;

    Ok(())
}

fn process_records() -> Result<(), Box<dyn std::error::Error>> {
    let records = vec![
        Record {
            id: 1,
            name: "Alpha".to_string(),
            value: 42.5,
        },
        Record {
            id: 2,
            name: "Beta".to_string(),
            value: 33.7,
        },
    ];

    let output_file = File::create("output.json")?;
    let writer = BufWriter::new(output_file);
    serde_json::to_writer_pretty(writer, &records)?;

    println!("Processed {} records", records.len());
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_files = vec![
        "data1.json".to_string(),
        "data2.json".to_string(),
    ];
    
    merge_json_files(&input_files, "merged.json", "id")?;
    process_records()?;
    
    Ok(())
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
                merged_map.insert(key, value);
            }
        } else {
            eprintln!("Warning: '{}' does not contain a JSON object, skipping.", input_path);
        }
    }

    let output_file = File::create(output_path)?;
    let pretty_json = serde_json::to_string_pretty(&merged_map)?;
    write!(&output_file, "{}", pretty_json)?;

    println!("Successfully merged {} file(s) into '{}'", input_paths.len(), output_path);
    Ok(())
}use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(
    input_paths: &[P],
    output_path: P,
    deduplicate_by_key: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_keys = HashSet::new();

    for path in input_paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let value: Value = serde_json::from_reader(reader)?;

        match value {
            Value::Array(arr) => {
                for item in arr {
                    if let Some(key) = deduplicate_by_key {
                        if let Some(obj) = item.as_object() {
                            if let Some(key_value) = obj.get(key) {
                                let key_str = key_value.to_string();
                                if !seen_keys.contains(&key_str) {
                                    seen_keys.insert(key_str);
                                    merged_array.push(item);
                                }
                                continue;
                            }
                        }
                    }
                    merged_array.push(item);
                }
            }
            _ => merged_array.push(value),
        }
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &json!(merged_array))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_basic() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"[{"id": 1}, {"id": 2}]"#).unwrap();
        fs::write(&file2, r#"[{"id": 3}, {"id": 4}]"#).unwrap();

        merge_json_files(&[file1.path(), file2.path()], output.path(), None).unwrap();

        let content = fs::read_to_string(output.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 4);
    }

    #[test]
    fn test_deduplicate() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"[{"id": 1, "name": "a"}, {"id": 2, "name": "b"}]"#).unwrap();
        fs::write(&file2, r#"[{"id": 1, "name": "c"}, {"id": 3, "name": "d"}]"#).unwrap();

        merge_json_files(&[file1.path(), file2.path()], output.path(), Some("id")).unwrap();

        let content = fs::read_to_string(output.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }
}