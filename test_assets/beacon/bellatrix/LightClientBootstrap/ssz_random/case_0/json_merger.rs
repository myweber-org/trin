use serde_json::{Value, Map};
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
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_merge_json_files() {
        let dir = tempdir().unwrap();
        
        let file1_path = dir.path().join("a.json");
        let mut file1 = File::create(&file1_path).unwrap();
        writeln!(file1, r#"{"name": "test", "count": 42}"#).unwrap();

        let file2_path = dir.path().join("b.json");
        let mut file2 = File::create(&file2_path).unwrap();
        writeln!(file2, r#"{"active": true, "tags": ["rust", "json"]}"#).unwrap();

        let paths = [
            file1_path.to_str().unwrap(),
            file2_path.to_str().unwrap(),
            "non_existent.json"
        ];

        let result = merge_json_files(&paths).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "test");
        assert_eq!(obj.get("count").unwrap().as_i64().unwrap(), 42);
        assert_eq!(obj.get("active").unwrap().as_bool().unwrap(), true);
        assert!(obj.get("non_existent").is_none());
    }
}