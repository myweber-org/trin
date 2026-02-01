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
}use serde_json::{Map, Value};
use std::collections::HashSet;

pub fn merge_json(base: &mut Value, update: &Value, overwrite_arrays: bool) {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (key, update_value) in update_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    merge_json(base_value, update_value, overwrite_arrays);
                } else {
                    base_map.insert(key.clone(), update_value.clone());
                }
            }
        }
        (Value::Array(base_arr), Value::Array(update_arr)) if !overwrite_arrays => {
            let mut existing_set = HashSet::new();
            for item in base_arr.iter() {
                if let Some(s) = item.as_str() {
                    existing_set.insert(s.to_string());
                }
            }
            
            for item in update_arr {
                if let Some(s) = item.as_str() {
                    if !existing_set.contains(s) {
                        base_arr.push(Value::String(s.to_string()));
                    }
                } else {
                    base_arr.push(item.clone());
                }
            }
        }
        (base, update) => {
            *base = update.clone();
        }
    }
}

pub fn merge_json_with_strategy(
    base: &str,
    update: &str,
    overwrite_arrays: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut base_value: Value = serde_json::from_str(base)?;
    let update_value: Value = serde_json::from_str(update)?;
    
    merge_json(&mut base_value, &update_value, overwrite_arrays);
    
    Ok(serde_json::to_string_pretty(&base_value)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_merge() {
        let base = r#"{"name": "Alice", "age": 30}"#;
        let update = r#"{"age": 31, "city": "New York"}"#;
        
        let result = merge_json_with_strategy(base, update, false).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        
        assert_eq!(parsed["name"], "Alice");
        assert_eq!(parsed["age"], 31);
        assert_eq!(parsed["city"], "New York");
    }
    
    #[test]
    fn test_nested_merge() {
        let base = r#"{"user": {"name": "Bob", "settings": {"theme": "dark"}}}"#;
        let update = r#"{"user": {"settings": {"language": "en"}}}"#;
        
        let result = merge_json_with_strategy(base, update, false).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        
        assert_eq!(parsed["user"]["name"], "Bob");
        assert_eq!(parsed["user"]["settings"]["theme"], "dark");
        assert_eq!(parsed["user"]["settings"]["language"], "en");
    }
}