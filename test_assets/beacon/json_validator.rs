rust
use serde_json::{Error, Value};
use std::fs;

pub struct JsonValidator;

impl JsonValidator {
    pub fn validate_string(json_str: &str) -> Result<Value, Error> {
        serde_json::from_str(json_str)
    }

    pub fn validate_file(file_path: &str) -> Result<Value, Error> {
        let content = fs::read_to_string(file_path)?;
        Self::validate_string(&content)
    }

    pub fn is_valid(json_str: &str) -> bool {
        Self::validate_string(json_str).is_ok()
    }

    pub fn get_validation_details(json_str: &str) -> Result<ValidationDetails, Error> {
        let value = Self::validate_string(json_str)?;
        
        let details = ValidationDetails {
            is_valid: true,
            total_keys: count_keys(&value),
            depth: calculate_depth(&value),
            data_type: get_root_type(&value),
        };
        
        Ok(details)
    }
}

fn count_keys(value: &Value) -> usize {
    match value {
        Value::Object(map) => {
            let mut count = map.len();
            for (_, v) in map {
                count += count_keys(v);
            }
            count
        }
        Value::Array(arr) => {
            let mut count = 0;
            for v in arr {
                count += count_keys(v);
            }
            count
        }
        _ => 0,
    }
}

fn calculate_depth(value: &Value) -> usize {
    match value {
        Value::Object(map) => {
            if map.is_empty() {
                1
            } else {
                1 + map.values().map(|v| calculate_depth(v)).max().unwrap_or(0)
            }
        }
        Value::Array(arr) => {
            if arr.is_empty() {
                1
            } else {
                1 + arr.iter().map(|v| calculate_depth(v)).max().unwrap_or(0)
            }
        }
        _ => 1,
    }
}

fn get_root_type(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(_) => "boolean".to_string(),
        Value::Number(_) => "number".to_string(),
        Value::String(_) => "string".to_string(),
        Value::Array(_) => "array".to_string(),
        Value::Object(_) => "object".to_string(),
    }
}

#[derive(Debug, Clone)]
pub struct ValidationDetails {
    pub is_valid: bool,
    pub total_keys: usize,
    pub depth: usize,
    pub data_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let valid_json = r#"{"name": "test", "value": 42}"#;
        assert!(JsonValidator::is_valid(valid_json));
        
        let result = JsonValidator::validate_string(valid_json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_json() {
        let invalid_json = r#"{"name": "test", "value": 42"#;
        assert!(!JsonValidator::is_valid(invalid_json));
        
        let result = JsonValidator::validate_string(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_details() {
        let json = r#"{"user": {"name": "john", "age": 30}, "active": true}"#;
        let details = JsonValidator::get_validation_details(json).unwrap();
        
        assert!(details.is_valid);
        assert_eq!(details.data_type, "object");
        assert_eq!(details.total_keys, 3);
        assert_eq!(details.depth, 2);
    }
}
```