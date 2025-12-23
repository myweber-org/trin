
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut result = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            merge_objects(&mut result, obj);
        }
    }

    Ok(Value::Object(result))
}

fn merge_objects(target: &mut Map<String, Value>, source: Map<String, Value>) {
    for (key, source_value) in source {
        match target.get_mut(&key) {
            Some(target_value) => {
                if let (Value::Object(mut target_obj), Value::Object(source_obj)) = (target_value.clone(), source_value.clone()) {
                    merge_objects(&mut target_obj, source_obj);
                    target.insert(key, Value::Object(target_obj));
                } else if target_value != &source_value {
                    let merged_array = match (target_value, &source_value) {
                        (Value::Array(arr1), Value::Array(arr2)) => {
                            let mut combined = arr1.clone();
                            combined.extend(arr2.clone());
                            Value::Array(combined)
                        }
                        _ => Value::Array(vec![target_value.clone(), source_value.clone()]),
                    };
                    target.insert(key, merged_array);
                }
            }
            None => {
                target.insert(key, source_value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() -> Result<(), Box<dyn std::error::Error>> {
        let file1 = NamedTempFile::new()?;
        let file2 = NamedTempFile::new()?;

        fs::write(&file1, r#"{"a": 1, "b": {"c": 2}}"#)?;
        fs::write(&file2, r#"{"b": {"d": 3}, "e": 4}"#)?;

        let merged = merge_json_files(&[file1.path(), file2.path()])?;
        let expected = json!({
            "a": 1,
            "b": {"c": 2, "d": 3},
            "e": 4
        });

        assert_eq!(merged, expected);
        Ok(())
    }
}