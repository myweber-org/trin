
use serde_json::{Map, Value};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

pub fn merge_json_files(input_paths: &[impl AsRef<Path>], output_path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for input_path in input_paths {
        let file = File::open(input_path.as_ref())?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Input JSON is not an object".into());
        }
    }

    let merged_value = Value::Object(merged_map);
    let mut output_file = File::create(output_path.as_ref())?;
    write!(output_file, "{}", merged_value.to_string())?;

    Ok(())
}