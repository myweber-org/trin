use serde_json::{Value, Error};

pub fn validate_json(json_str: &str) -> Result<Value, String> {
    match serde_json::from_str(json_str) {
        Ok(parsed) => Ok(parsed),
        Err(e) => Err(format!("Invalid JSON: {}", e)),
    }
}

pub fn is_valid_json(json_str: &str) -> bool {
    validate_json(json_str).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let valid_json = r#"{"name": "test", "value": 42}"#;
        assert!(is_valid_json(valid_json));
        assert!(validate_json(valid_json).is_ok());
    }

    #[test]
    fn test_invalid_json() {
        let invalid_json = r#"{"name": "test", "value": }"#;
        assert!(!is_valid_json(invalid_json));
        assert!(validate_json(invalid_json).is_err());
    }

    #[test]
    fn test_empty_string() {
        assert!(!is_valid_json(""));
        assert!(validate_json("").is_err());
    }
}