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