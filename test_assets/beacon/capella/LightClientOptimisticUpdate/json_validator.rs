use serde_json::{Error, Value};
use std::fs;

pub fn validate_json_from_str(json_str: &str) -> Result<Value, Error> {
    serde_json::from_str(json_str)
}

pub fn validate_json_from_file(file_path: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let parsed = serde_json::from_str(&content)?;
    Ok(parsed)
}

pub fn is_valid_json(json_str: &str) -> bool {
    validate_json_from_str(json_str).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let valid_json = r#"{"name": "test", "value": 42}"#;
        assert!(is_valid_json(valid_json));
    }

    #[test]
    fn test_invalid_json() {
        let invalid_json = r#"{"name": test, "value": 42}"#;
        assert!(!is_valid_json(invalid_json));
    }

    #[test]
    fn test_validate_returns_value() {
        let json_str = r#"{"data": [1, 2, 3]}"#;
        let result = validate_json_from_str(json_str);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value["data"].is_array());
    }
}