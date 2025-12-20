use serde_json::{Value, from_reader, to_writer_pretty};
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader};
use std::path::Path;

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> io::Result<()> {
    let mut merged_array = Vec::new();

    for path_str in input_paths {
        let path = Path::new(path_str);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = from_reader(reader)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        if let Value::Array(arr) = json_value {
            merged_array.extend(arr);
        } else {
            merged_array.push(json_value);
        }
    }

    let output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_path)?;

    to_writer_pretty(output_file, &Value::Array(merged_array))
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    Ok(())
}