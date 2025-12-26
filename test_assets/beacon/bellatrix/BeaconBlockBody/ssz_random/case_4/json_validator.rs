use serde_json::{Error, Value};
use std::fs;

pub fn validate_json_file(file_path: &str) -> Result<Value, String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let parsed: Result<Value, Error> = serde_json::from_str(&content);

    match parsed {
        Ok(json_value) => Ok(json_value),
        Err(e) => Err(format!("Invalid JSON: {}", e)),
    }
}

pub fn validate_json_string(json_str: &str) -> Result<Value, String> {
    let parsed: Result<Value, Error> = serde_json::from_str(json_str);

    match parsed {
        Ok(json_value) => Ok(json_value),
        Err(e) => Err(format!("Invalid JSON: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json_string() {
        let valid_json = r#"{"name": "test", "value": 42}"#;
        let result = validate_json_string(valid_json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_json_string() {
        let invalid_json = r#"{"name": "test", "value": }"#;
        let result = validate_json_string(invalid_json);
        assert!(result.is_err());
    }
}