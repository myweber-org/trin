use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for file_path in file_paths {
        let content = fs::read_to_string(file_path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    let merged_json = json!(merged_map);
    let pretty_json = serde_json::to_string_pretty(&merged_json)?;

    let output_dir = Path::new(output_path).parent().unwrap();
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    fs::write(output_path, pretty_json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1_content = r#"{"a": 1, "b": 2}"#;
        let file2_content = r#"{"c": 3, "d": 4}"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), file1_content).unwrap();
        fs::write(file2.path(), file2_content).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        merge_json_files(&paths, output_file.path().to_str().unwrap()).unwrap();

        let result_content = fs::read_to_string(output_file.path()).unwrap();
        let result_value: Value = serde_json::from_str(&result_content).unwrap();

        assert_eq!(result_value["a"], 1);
        assert_eq!(result_value["b"], 2);
        assert_eq!(result_value["c"], 3);
        assert_eq!(result_value["d"], 4);
    }
}