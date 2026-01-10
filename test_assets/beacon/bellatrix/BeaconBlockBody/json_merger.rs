
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufReader, Write};
use std::path::Path;

use serde_json::{Value, json};

pub fn merge_json_files<P: AsRef<Path>>(input_paths: &[P], output_path: P) -> io::Result<()> {
    let mut merged_array = Vec::new();
    let mut seen_objects = HashMap::new();

    for input_path in input_paths {
        let file = File::open(input_path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    add_unique_object(&mut merged_array, &mut seen_objects, item);
                }
            }
            Value::Object(_) => {
                add_unique_object(&mut merged_array, &mut seen_objects, json_value);
            }
            _ => {
                eprintln!("Warning: Skipping non-object/non-array JSON in {:?}", input_path.as_ref());
            }
        }
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &json!(merged_array))?;
    Ok(())
}

fn add_unique_object(array: &mut Vec<Value>, seen: &mut HashMap<String, bool>, obj: Value) {
    if let Some(id) = extract_object_id(&obj) {
        if !seen.contains_key(&id) {
            seen.insert(id.clone(), true);
            array.push(obj);
        }
    } else {
        array.push(obj);
    }
}

fn extract_object_id(obj: &Value) -> Option<String> {
    obj.get("id")
        .or_else(|| obj.get("uuid"))
        .or_else(|| obj.get("_id"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use serde_json::json;

    #[test]
    fn test_merge_json_arrays() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        let data1 = json!([{"id": "a", "value": 1}, {"id": "b", "value": 2}]);
        let data2 = json!([{"id": "b", "value": 3}, {"id": "c", "value": 4}]);

        fs::write(file1.path(), data1.to_string()).unwrap();
        fs::write(file2.path(), data2.to_string()).unwrap();

        let inputs = [file1.path(), file2.path()];
        merge_json_files(&inputs, output_file.path()).unwrap();

        let content = fs::read_to_string(output_file.path()).unwrap();
        let result: Value = serde_json::from_str(&content).unwrap();

        assert_eq!(result.as_array().unwrap().len(), 3);
        assert!(result.as_array().unwrap().iter().any(|v| v["id"] == "a"));
        assert!(result.as_array().unwrap().iter().any(|v| v["id"] == "b"));
        assert!(result.as_array().unwrap().iter().any(|v| v["id"] == "c"));
    }
}