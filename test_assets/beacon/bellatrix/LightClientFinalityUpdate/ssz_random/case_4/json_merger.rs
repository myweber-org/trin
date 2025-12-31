use serde_json::{json, Value};
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(input_paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for path in input_paths {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: Value = serde_json::from_str(&contents)?;
        merged_array.push(json_value);
    }

    let output_json = json!(merged_array);
    let mut output_file = File::create(output_path)?;
    write!(output_file, "{}", output_json.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        writeln!(file1.as_file(), r#"{{"id": 1, "name": "Alice"}}"#).unwrap();
        writeln!(file2.as_file(), r#"{{"id": 2, "name": "Bob"}}"#).unwrap();

        let inputs = [file1.path(), file2.path()];
        merge_json_files(&inputs, output_file.path()).unwrap();

        let output_contents = std::fs::read_to_string(output_file.path()).unwrap();
        let expected = r#"[{"id":1,"name":"Alice"},{"id":2,"name":"Bob"}]"#;
        assert_eq!(output_contents, expected);
    }
}