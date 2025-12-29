use serde_json::{Value, from_reader, to_writer_pretty};
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader};
use std::path::Path;

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> io::Result<()> {
    let mut merged_array = Vec::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        if !path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("File not found: {}", input_path),
            ));
        }

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

    to_writer_pretty(output_file, &merged_array)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        let data1 = json!([{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]);
        let data2 = json!([{"id": 3, "name": "Charlie"}]);

        fs::write(file1.path(), data1.to_string()).unwrap();
        fs::write(file2.path(), data2.to_string()).unwrap();

        let input_paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        merge_json_files(&input_paths, output_file.path().to_str().unwrap()).unwrap();

        let content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        assert_eq!(parsed.as_array().unwrap().len(), 3);
        assert_eq!(parsed[0]["name"], "Alice");
        assert_eq!(parsed[2]["name"], "Charlie");
    }
}