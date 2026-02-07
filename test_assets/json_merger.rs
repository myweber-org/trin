
use serde_json::{Value, json};
use std::fs::{self, File};
use std::io::{BufReader, Result};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<()> {
    let mut merged_array = Vec::new();

    for path in paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let value: Value = serde_json::from_reader(reader)?;

        if let Value::Array(arr) = value {
            merged_array.extend(arr);
        } else {
            merged_array.push(value);
        }
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &json!(merged_array))?;
    Ok(())
}