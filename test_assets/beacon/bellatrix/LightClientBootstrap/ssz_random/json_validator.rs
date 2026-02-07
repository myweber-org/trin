use serde_json::{Value, from_str};
use std::collections::HashSet;

pub struct JsonValidator {
    required_fields: HashSet<String>,
    allowed_types: HashSet<&'static str>,
}

impl JsonValidator {
    pub fn new() -> Self {
        JsonValidator {
            required_fields: HashSet::new(),
            allowed_types: HashSet::from(["string", "number", "boolean", "object", "array"]),
        }
    }

    pub fn add_required_field(&mut self, field: &str) {
        self.required_fields.insert(field.to_string());
    }

    pub fn validate(&self, json_str: &str) -> Result<Value, String> {
        let parsed: Value = from_str(json_str)
            .map_err(|e| format!("Invalid JSON: {}", e))?;

        self.validate_structure(&parsed)?;
        Ok(parsed)
    }

    fn validate_structure(&self, value: &Value) -> Result<(), String> {
        match value {
            Value::Object(map) => {
                for field in &self.required_fields {
                    if !map.contains_key(field) {
                        return Err(format!("Missing required field: {}", field));
                    }
                }

                for (key, val) in map {
                    self.validate_value_type(val, key)?;
                }
                Ok(())
            }
            _ => Err("Expected JSON object at root level".to_string()),
        }
    }

    fn validate_value_type(&self, value: &Value, path: &str) -> Result<(), String> {
        let type_name = match value {
            Value::String(_) => "string",
            Value::Number(_) => "number",
            Value::Bool(_) => "boolean",
            Value::Object(_) => "object",
            Value::Array(_) => "array",
            Value::Null => "null",
        };

        if !self.allowed_types.contains(type_name) {
            return Err(format!("Field '{}' has disallowed type: {}", path, type_name));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let mut validator = JsonValidator::new();
        validator.add_required_field("name");
        
        let json = r#"{"name": "test", "age": 25}"#;
        assert!(validator.validate(json).is_ok());
    }

    #[test]
    fn test_missing_required_field() {
        let mut validator = JsonValidator::new();
        validator.add_required_field("email");
        
        let json = r#"{"name": "test"}"#;
        assert!(validator.validate(json).is_err());
    }
}