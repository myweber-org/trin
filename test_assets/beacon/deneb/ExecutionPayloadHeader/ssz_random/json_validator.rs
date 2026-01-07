use serde_json::{Value, Error};
use std::collections::HashSet;

pub struct JsonValidator {
    required_fields: HashSet<String>,
    allowed_types: HashSet<&'static str>,
}

impl JsonValidator {
    pub fn new() -> Self {
        JsonValidator {
            required_fields: HashSet::new(),
            allowed_types: HashSet::from(["string", "number", "boolean", "object", "array", "null"]),
        }
    }

    pub fn add_required_field(&mut self, field: &str) {
        self.required_fields.insert(field.to_string());
    }

    pub fn validate(&self, json_str: &str) -> Result<Value, ValidationError> {
        let parsed: Value = serde_json::from_str(json_str)
            .map_err(|e| ValidationError::ParseError(e.to_string()))?;

        self.validate_structure(&parsed)?;
        self.validate_required_fields(&parsed)?;

        Ok(parsed)
    }

    fn validate_structure(&self, value: &Value) -> Result<(), ValidationError> {
        match value {
            Value::Object(map) => {
                for (_, v) in map {
                    self.validate_structure(v)?;
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    self.validate_structure(item)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn validate_required_fields(&self, value: &Value) -> Result<(), ValidationError> {
        if let Value::Object(map) = value {
            for field in &self.required_fields {
                if !map.contains_key(field) {
                    return Err(ValidationError::MissingField(field.clone()));
                }
            }
        }
        Ok(())
    }

    pub fn extract_string_field(&self, value: &Value, field: &str) -> Option<String> {
        value.get(field)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    pub fn extract_number_field(&self, value: &Value, field: &str) -> Option<f64> {
        value.get(field).and_then(|v| v.as_f64())
    }
}

#[derive(Debug)]
pub enum ValidationError {
    ParseError(String),
    MissingField(String),
    InvalidType(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::ParseError(msg) => write!(f, "JSON parse error: {}", msg),
            ValidationError::MissingField(field) => write!(f, "Missing required field: {}", field),
            ValidationError::InvalidType(msg) => write!(f, "Invalid type: {}", msg),
        }
    }
}

impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let mut validator = JsonValidator::new();
        validator.add_required_field("name");
        
        let json = r#"{"name": "test", "value": 42}"#;
        let result = validator.validate(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_missing_required_field() {
        let mut validator = JsonValidator::new();
        validator.add_required_field("name");
        
        let json = r#"{"value": 42}"#;
        let result = validator.validate(json);
        assert!(matches!(result, Err(ValidationError::MissingField(_))));
    }

    #[test]
    fn test_extract_fields() {
        let validator = JsonValidator::new();
        let json = r#"{"name": "test", "count": 100}"#;
        let value: Value = serde_json::from_str(json).unwrap();
        
        assert_eq!(validator.extract_string_field(&value, "name"), Some("test".to_string()));
        assert_eq!(validator.extract_number_field(&value, "count"), Some(100.0));
    }
}