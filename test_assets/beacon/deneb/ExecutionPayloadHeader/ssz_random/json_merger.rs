use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

type JsonValue = serde_json::Value;
type JsonResult = Result<JsonValue, Box<dyn std::error::Error>>;

pub struct JsonMerger {
    data: HashMap<String, JsonValue>,
}

impl JsonMerger {
    pub fn new() -> Self {
        JsonMerger {
            data: HashMap::new(),
        }
    }

    pub fn add_file<P: AsRef<Path>>(&mut self, path: P) -> JsonResult {
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: JsonValue = serde_json::from_str(&contents)?;
        let filename = path.as_ref()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        self.data.insert(filename, json_value);
        Ok(())
    }

    pub fn merge(&self) -> JsonValue {
        let mut merged = HashMap::new();
        for (key, value) in &self.data {
            merged.insert(key.clone(), value.clone());
        }
        JsonValue::Object(serde_json::Map::from_iter(merged.into_iter()))
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, output_path: P) -> std::io::Result<()> {
        let merged = self.merge();
        let json_string = serde_json::to_string_pretty(&merged)?;
        std::fs::write(output_path, json_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_json_merger() {
        let mut merger = JsonMerger::new();

        let file1_content = r#"{"name": "test", "value": 42}"#;
        let file2_content = r#"{"enabled": true, "tags": ["rust", "json"]}"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        std::fs::write(&file1, file1_content).unwrap();
        std::fs::write(&file2, file2_content).unwrap();

        merger.add_file(file1.path()).unwrap();
        merger.add_file(file2.path()).unwrap();

        let merged = merger.merge();
        assert!(merged.is_object());
        let obj = merged.as_object().unwrap();
        assert!(obj.contains_key("test"));
        assert!(obj.contains_key("file"));

        let output = NamedTempFile::new().unwrap();
        merger.save_to_file(output.path()).unwrap();
        assert!(output.path().exists());
    }
}
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

        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Key '{}' already exists, overwriting.", key);
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object at root level".into());
        }
    }

    Ok(serde_json::Value::Object(merged_map))
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

        writeln!(file1, r#"{"name": "Alice", "age": 30}"#).unwrap();
        writeln!(file2, r#"{"city": "London", "active": true}"#).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let result_obj = result.as_object().unwrap();

        assert_eq!(result_obj.get("name").unwrap().as_str().unwrap(), "Alice");
        assert_eq!(result_obj.get("age").unwrap().as_u64().unwrap(), 30);
        assert_eq!(result_obj.get("city").unwrap().as_str().unwrap(), "London");
        assert_eq!(result_obj.get("active").unwrap().as_bool().unwrap(), true);
        assert_eq!(result_obj.len(), 4);
    }

    #[test]
    fn test_duplicate_keys_overwrite() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"id": 100, "value": "first"}"#).unwrap();
        writeln!(file2, r#"{"id": 200, "extra": "data"}"#).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let result_obj = result.as_object().unwrap();

        assert_eq!(result_obj.get("id").unwrap().as_u64().unwrap(), 200);
        assert_eq!(result_obj.get("value").unwrap().as_str().unwrap(), "first");
        assert_eq!(result_obj.get("extra").unwrap().as_str().unwrap(), "data");
    }
}