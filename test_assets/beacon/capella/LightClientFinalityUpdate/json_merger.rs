
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

type JsonValue = serde_json::Value;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct JsonMerger {
    data: HashMap<String, JsonValue>,
}

impl JsonMerger {
    pub fn new() -> Self {
        JsonMerger {
            data: HashMap::new(),
        }
    }

    pub fn add_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_data: JsonValue = serde_json::from_str(&contents)?;
        
        if let JsonValue::Object(map) = json_data {
            for (key, value) in map {
                self.data.insert(key, value);
            }
        } else {
            return Err("Root JSON element must be an object".into());
        }

        Ok(())
    }

    pub fn add_files<P: AsRef<Path>>(&mut self, paths: &[P]) -> Result<()> {
        for path in paths {
            self.add_file(path)?;
        }
        Ok(())
    }

    pub fn merge(&self) -> JsonValue {
        let mut result_map = serde_json::Map::new();
        for (key, value) in &self.data {
            result_map.insert(key.clone(), value.clone());
        }
        JsonValue::Object(result_map)
    }

    pub fn merge_to_string(&self) -> Result<String> {
        let merged = self.merge();
        let json_string = serde_json::to_string_pretty(&merged)?;
        Ok(json_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_json(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn test_basic_merge() {
        let file1 = create_temp_json(r#"{"name": "test", "value": 42}"#);
        let file2 = create_temp_json(r#"{"enabled": true, "tags": ["a", "b"]}"#);

        let mut merger = JsonMerger::new();
        merger.add_file(file1.path()).unwrap();
        merger.add_file(file2.path()).unwrap();

        let merged = merger.merge();
        let merged_str = merger.merge_to_string().unwrap();

        assert!(merged.is_object());
        assert!(merged_str.contains("\"name\": \"test\""));
        assert!(merged_str.contains("\"value\": 42"));
        assert!(merged_str.contains("\"enabled\": true"));
    }

    #[test]
    fn test_overwrite_behavior() {
        let file1 = create_temp_json(r#"{"id": 1, "status": "old"}"#);
        let file2 = create_temp_json(r#"{"id": 2, "status": "new"}"#);

        let mut merger = JsonMerger::new();
        merger.add_file(file1.path()).unwrap();
        merger.add_file(file2.path()).unwrap();

        let merged = merger.merge();
        let obj = merged.as_object().unwrap();

        assert_eq!(obj.get("id").unwrap().as_i64().unwrap(), 2);
        assert_eq!(obj.get("status").unwrap().as_str().unwrap(), "new");
    }
}
use serde_json::{Map, Value};
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

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
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

        let json1 = json!({"name": "Alice", "age": 30});
        let json2 = json!({"city": "Berlin", "active": true});

        write!(file1, "{}", json1).unwrap();
        write!(file2, "{}", json2).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
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