use serde_json::{Value, Map};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(input_paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_ids = HashSet::new();

    for path in input_paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_array: Vec<Value> = serde_json::from_reader(reader)?;

        for item in json_array {
            if let Some(obj) = item.as_object() {
                if let Some(id_value) = obj.get("id") {
                    if let Some(id_str) = id_value.as_str() {
                        if !seen_ids.contains(id_str) {
                            seen_ids.insert(id_str.to_string());
                            merged_array.push(item);
                        }
                        continue;
                    }
                }
            }
            merged_array.push(item);
        }
    }

    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    serde_json::to_writer_pretty(writer, &merged_array)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let data1 = json!([{"id": "1", "name": "Alice"}, {"id": "2", "name": "Bob"}]);
        let data2 = json!([{"id": "2", "name": "Robert"}, {"id": "3", "name": "Charlie"}]);

        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        writeln!(file1, "{}", data1.to_string()).unwrap();
        writeln!(file2, "{}", data2.to_string()).unwrap();

        let inputs = [file1.path(), file2.path()];
        merge_json_files(&inputs, output_file.path()).unwrap();

        let output_content = std::fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();

        assert_eq!(parsed.as_array().unwrap().len(), 3);
        assert!(parsed.as_array().unwrap().iter().any(|v| v["id"] == "1"));
        assert!(parsed.as_array().unwrap().iter().any(|v| v["id"] == "2"));
        assert!(parsed.as_array().unwrap().iter().any(|v| v["id"] == "3"));
    }
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged = Map::new();
    
    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;
        
        if let Value::Object(obj) = json {
            merge_objects(&mut merged, obj);
        }
    }
    
    let output_json = Value::Object(merged);
    let output_str = serde_json::to_string_pretty(&output_json)?;
    fs::write(output_path, output_str)?;
    
    Ok(())
}

fn merge_objects(target: &mut Map<String, Value>, source: Map<String, Value>) {
    for (key, value) in source {
        if let Some(existing) = target.get_mut(&key) {
            if let (Value::Object(mut target_obj), Value::Object(source_obj)) = (existing, value) {
                merge_objects(&mut target_obj, source_obj);
                *existing = Value::Object(target_obj);
            } else {
                *existing = value;
            }
        } else {
            target.insert(key, value);
        }
    }
}use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(Value::Object(merged_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        let json1 = json!({
            "name": "Alice",
            "age": 30
        });
        let json2 = json!({
            "city": "Berlin",
            "active": true
        });

        write!(file1, "{}", json1.to_string()).unwrap();
        write!(file2, "{}", json2.to_string()).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
            "non_existent.json",
        ];

        let result = merge_json_files(&paths).unwrap();
        let expected = json!({
            "name": "Alice",
            "age": 30,
            "city": "Berlin",
            "active": true
        });

        assert_eq!(result, expected);
    }
}