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
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str).into());
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                merged.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    Ok(serde_json::Value::Object(
        merged
            .into_iter()
            .map(|(k, v)| (k, v))
            .collect()
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
    fn test_merge_with_duplicate_keys() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{{ "id": 1, "value": "first" }}"#).unwrap();
        writeln!(file2, r#"{{ "id": 2, "extra": "data" }}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]);

        assert!(result.is_ok());
        let merged = result.unwrap();
        assert_eq!(merged["id"], 2);
        assert_eq!(merged["value"], "first");
        assert_eq!(merged["extra"], "data");
    }
}use serde_json::{Map, Value};

pub fn merge_json(a: &mut Value, b: &Value) {
    match (a, b) {
        (Value::Object(a_obj), Value::Object(b_obj)) => {
            for (key, b_val) in b_obj {
                if let Some(a_val) = a_obj.get_mut(key) {
                    merge_json(a_val, b_val);
                } else {
                    a_obj.insert(key.clone(), b_val.clone());
                }
            }
        }
        (a_val, b_val) => {
            *a_val = b_val.clone();
        }
    }
}

pub fn merge_multiple_json(objects: Vec<Value>) -> Option<Value> {
    let mut result = Map::new().into();
    for obj in objects {
        merge_json(&mut result, &obj);
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_merge() {
        let mut a = json!({"name": "Alice", "age": 30});
        let b = json!({"age": 31, "city": "London"});
        merge_json(&mut a, &b);
        assert_eq!(a, json!({"name": "Alice", "age": 31, "city": "London"}));
    }

    #[test]
    fn test_nested_merge() {
        let mut a = json!({"user": {"name": "Alice", "prefs": {"theme": "dark"}}});
        let b = json!({"user": {"prefs": {"language": "en"}, "active": true}});
        merge_json(&mut a, &b);
        let expected = json!({
            "user": {
                "name": "Alice",
                "prefs": {"theme": "dark", "language": "en"},
                "active": true
            }
        });
        assert_eq!(a, expected);
    }

    #[test]
    fn test_multiple_merge() {
        let objects = vec![
            json!({"a": 1}),
            json!({"b": 2}),
            json!({"a": 3, "c": 4}),
        ];
        let result = merge_multiple_json(objects).unwrap();
        assert_eq!(result, json!({"a": 3, "b": 2, "c": 4}));
    }
}