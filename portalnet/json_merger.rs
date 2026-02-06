use serde_json::{Value, from_reader, to_writer_pretty};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Result};
use std::path::Path;

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<()> {
    let mut merged_array = Vec::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        if !path.exists() {
            eprintln!("Warning: File {} does not exist, skipping.", input_path);
            continue;
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = from_reader(reader)?;

        match json_value {
            Value::Array(arr) => {
                merged_array.extend(arr);
            }
            _ => {
                merged_array.push(json_value);
            }
        }
    }

    let output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_path)?;

    to_writer_pretty(output_file, &merged_array)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    #[test]
    fn test_merge_json_files() {
        let file1_content = json!([{"id": 1}, {"id": 2}]);
        let file2_content = json!([{"id": 3}, {"id": 4}]);
        let file3_content = json!({"id": 5});

        fs::write("test1.json", file1_content.to_string()).unwrap();
        fs::write("test2.json", file2_content.to_string()).unwrap();
        fs::write("test3.json", file3_content.to_string()).unwrap();

        let inputs = ["test1.json", "test2.json", "test3.json"];
        merge_json_files(&inputs, "merged_output.json").unwrap();

        let merged_content: Value = from_reader(File::open("merged_output.json").unwrap()).unwrap();
        let expected = json!([{"id": 1}, {"id": 2}, {"id": 3}, {"id": 4}, {"id": 5}]);

        assert_eq!(merged_content, expected);

        fs::remove_file("test1.json").unwrap();
        fs::remove_file("test2.json").unwrap();
        fs::remove_file("test3.json").unwrap();
        fs::remove_file("merged_output.json").unwrap();
    }
}