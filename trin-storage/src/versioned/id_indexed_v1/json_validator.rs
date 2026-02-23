use serde_json::{self, Value};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum JsonValidationError {
    ParseError(String),
    SchemaMismatch(String),
}

impl fmt::Display for JsonValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonValidationError::ParseError(msg) => write!(f, "JSON parse error: {}", msg),
            JsonValidationError::SchemaMismatch(msg) => write!(f, "Schema mismatch: {}", msg),
        }
    }
}

impl Error for JsonValidationError {}

pub struct JsonValidator {
    required_fields: Vec<String>,
    allowed_types: Vec<String>,
}

impl JsonValidator {
    pub fn new(required_fields: Vec<String>, allowed_types: Vec<String>) -> Self {
        JsonValidator {
            required_fields,
            allowed_types,
        }
    }

    pub fn validate(&self, json_str: &str) -> Result<Value, JsonValidationError> {
        let parsed: Value = serde_json::from_str(json_str)
            .map_err(|e| JsonValidationError::ParseError(e.to_string()))?;

        self.validate_structure(&parsed)?;
        Ok(parsed)
    }

    fn validate_structure(&self, value: &Value) -> Result<(), JsonValidationError> {
        if let Value::Object(map) = value {
            for field in &self.required_fields {
                if !map.contains_key(field) {
                    return Err(JsonValidationError::SchemaMismatch(
                        format!("Missing required field: {}", field)
                    ));
                }
            }

            for (key, val) in map {
                if !self.allowed_types.is_empty() {
                    let type_str = match val {
                        Value::Null => "null",
                        Value::Bool(_) => "boolean",
                        Value::Number(_) => "number",
                        Value::String(_) => "string",
                        Value::Array(_) => "array",
                        Value::Object(_) => "object",
                    };

                    if !self.allowed_types.contains(&type_str.to_string()) {
                        return Err(JsonValidationError::SchemaMismatch(
                            format!("Field '{}' has disallowed type: {}", key, type_str)
                        ));
                    }
                }
            }
        } else {
            return Err(JsonValidationError::SchemaMismatch(
                "Expected JSON object at root level".to_string()
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json_validation() {
        let validator = JsonValidator::new(
            vec!["name".to_string(), "age".to_string()],
            vec!["string".to_string(), "number".to_string()]
        );

        let json_data = r#"{"name": "Alice", "age": 30}"#;
        let result = validator.validate(json_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_missing_required_field() {
        let validator = JsonValidator::new(
            vec!["name".to_string(), "age".to_string()],
            vec![]
        );

        let json_data = r#"{"name": "Bob"}"#;
        let result = validator.validate(json_data);
        assert!(matches!(result, Err(JsonValidationError::SchemaMismatch(_))));
    }

    #[test]
    fn test_invalid_json_syntax() {
        let validator = JsonValidator::new(vec![], vec![]);
        let json_data = r#"{"name": "Charlie", "age": }"#;
        let result = validator.validate(json_data);
        assert!(matches!(result, Err(JsonValidationError::ParseError(_))));
    }
}