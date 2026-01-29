use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str).into());
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        if let serde_json::Value::Object(obj) = json_value {
            for (key, value) in obj {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    Ok(serde_json::Value::Object(
        merged_map
            .into_iter()
            .collect::<serde_json::Map<String, serde_json::Value>>(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{{ "name": "Alice", "age": 30 }}"#).unwrap();
        writeln!(file2, r#"{{ "city": "London", "active": true }}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]);

        assert!(result.is_ok());
        let merged = result.unwrap();
        assert_eq!(merged["name"], "Alice");
        assert_eq!(merged["age"], 30);
        assert_eq!(merged["city"], "London");
        assert_eq!(merged["active"], true);
    }

    #[test]
    fn test_merge_with_conflict() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{{ "key": "first" }}"#).unwrap();
        writeln!(file2, r#"{{ "key": "second" }}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]);

        assert!(result.is_ok());
        let merged = result.unwrap();
        assert_eq!(merged["key"], "second");
    }
}
use serde_json::{Value, from_reader, to_writer_pretty};
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(input_paths: &[P], output_path: P) -> io::Result<()> {
    let mut merged_array = Vec::new();

    for input_path in input_paths {
        let file = File::open(input_path)?;
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
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    Ok(())
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
        let output_file = NamedTempFile::new().unwrap();

        let json1 = json!([{"id": 1}, {"id": 2}]);
        let json2 = json!([{"id": 3}, {"id": 4}]);

        write!(file1, "{}", json1).unwrap();
        write!(file2, "{}", json2).unwrap();

        merge_json_files(
            &[file1.path(), file2.path()],
            output_file.path(),
        ).unwrap();

        let result_file = File::open(output_file.path()).unwrap();
        let reader = BufReader::new(result_file);
        let result: Value = from_reader(reader).unwrap();

        assert_eq!(result, json!([{"id": 1}, {"id": 2}, {"id": 3}, {"id": 4}]));
    }

    #[test]
    fn test_merge_mixed_json_types() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        let json1 = json!({"name": "Alice"});
        let json2 = json!([{"name": "Bob"}, {"name": "Charlie"}]);

        write!(file1, "{}", json1).unwrap();
        write!(file2, "{}", json2).unwrap();

        merge_json_files(
            &[file1.path(), file2.path()],
            output_file.path(),
        ).unwrap();

        let result_file = File::open(output_file.path()).unwrap();
        let reader = BufReader::new(result_file);
        let result: Value = from_reader(reader).unwrap();

        assert_eq!(result, json!([{"name": "Alice"}, {"name": "Bob"}, {"name": "Charlie"}]));
    }
}