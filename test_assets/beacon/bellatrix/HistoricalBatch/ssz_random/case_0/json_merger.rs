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

    pub fn load_file<P: AsRef<Path>>(&mut self, path: P) -> JsonResult {
        let file = File::open(&path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: JsonValue = serde_json::from_str(&contents)?;
        let filename = path
            .as_ref()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        self.data.insert(filename, json_value);
        Ok(())
    }

    pub fn load_directory<P: AsRef<Path>>(&mut self, dir_path: P) -> JsonResult {
        let entries = std::fs::read_dir(dir_path)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "json" {
                        self.load_file(&path)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn merge(&self) -> JsonValue {
        let mut merged = JsonValue::Object(serde_json::Map::new());
        for (key, value) in &self.data {
            merged[key] = value.clone();
        }
        merged
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, output_path: P) -> JsonResult {
        let merged = self.merge();
        let json_string = serde_json::to_string_pretty(&merged)?;
        std::fs::write(output_path, json_string)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_json_merger() {
        let temp_dir = TempDir::new().unwrap();
        let file1_path = temp_dir.path().join("config.json");
        let file2_path = temp_dir.path().join("data.json");
        let output_path = temp_dir.path().join("merged.json");

        std::fs::write(&file1_path, r#"{"version": "1.0.0"}"#).unwrap();
        std::fs::write(&file2_path, r#"{"items": [1, 2, 3]}"#).unwrap();

        let mut merger = JsonMerger::new();
        merger.load_file(&file1_path).unwrap();
        merger.load_file(&file2_path).unwrap();

        merger.save_to_file(&output_path).unwrap();

        let merged_content = std::fs::read_to_string(&output_path).unwrap();
        let parsed: JsonValue = serde_json::from_str(&merged_content).unwrap();

        assert!(parsed.get("config").is_some());
        assert!(parsed.get("data").is_some());
    }
}