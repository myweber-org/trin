
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