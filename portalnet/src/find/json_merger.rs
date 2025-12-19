use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], deduplicate_by_key: Option<&str>) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_keys = HashSet::new();

    for path in paths {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: Value = serde_json::from_str(&contents)?;

        match json_value {
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
            _ => merged_array.push(json_value),
        }
    }

    Ok(json!(merged_array))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_arrays() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        let data1 = json!([{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]);
        let data2 = json!([{"id": 3, "name": "Charlie"}]);

        write!(file1, "{}", data1).unwrap();
        write!(file2, "{}", data2).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()], None).unwrap();
        assert_eq!(result.as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_deduplicate_by_key() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        let data1 = json!([{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]);
        let data2 = json!([{"id": 2, "name": "Robert"}, {"id": 3, "name": "Charlie"}]);

        write!(file1, "{}", data1).unwrap();
        write!(file2, "{}", data2).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()], Some("id")).unwrap();
        let array = result.as_array().unwrap();
        assert_eq!(array.len(), 3);
        
        let ids: Vec<i64> = array
            .iter()
            .filter_map(|v| v["id"].as_i64())
            .collect();
        assert_eq!(ids, vec![1, 2, 3]);
    }
}