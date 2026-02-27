use serde_json::{Value, from_reader, to_writer_pretty};
use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> io::Result<()> {
    let mut merged_array = Vec::new();

    for path in paths {
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

    let output_file = File::create(output_path)?;
    to_writer_pretty(output_file, &Value::Array(merged_array))
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    Ok(())
}

pub fn merge_json_files_in_directory<P: AsRef<Path>>(dir_path: P, output_path: P) -> io::Result<()> {
    let mut json_paths = Vec::new();

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "json") {
            json_paths.push(path);
        }
    }

    if json_paths.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No JSON files found in directory",
        ));
    }

    merge_json_files(&json_paths, output_path)
}