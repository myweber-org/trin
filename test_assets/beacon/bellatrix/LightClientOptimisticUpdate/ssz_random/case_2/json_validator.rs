use serde_json::{Value, Map};
use std::collections::HashSet;
use std::error::Error;

pub struct JsonValidator {
    required_fields: HashSet<String>,
    allowed_types: Map<String, Vec<String>>,
}

impl JsonValidator {
    pub fn new() -> Self {
        JsonValidator {
            required_fields: HashSet::new(),
            allowed_types: Map::new(),
        }
    }

    pub fn add_required_field(&mut self, field: &str) {
        self.required_fields.insert(field.to_string());
    }

    pub fn add_allowed_type(&mut self, field: &str, types: Vec<&str>) {
        let type_strings: Vec<String> = types.iter().map(|s| s.to_string()).collect();
        self.allowed_types.insert(field.to_string(), type_strings);
    }

    pub fn validate(&self, json_str: &str) -> Result<(), Box<dyn Error>> {
        let value: Value = serde_json::from_str(json_str)?;
        
        if let Value::Object(obj) = &value {
            for field in &self.required_fields {
                if !obj.contains_key(field) {
                    return Err(format!("Missing required field: {}", field).into());
                }
            }

            for (field, allowed) in &self.allowed_types {
                if let Some(field_value) = obj.get(field) {
                    let actual_type = match field_value {
                        Value::Null => "null",
                        Value::Bool(_) => "boolean",
                        Value::Number(_) => "number",
                        Value::String(_) => "string",
                        Value::Array(_) => "array",
                        Value::Object(_) => "object",
                    };

                    if !allowed.contains(&actual_type.to_string()) {
                        return Err(format!("Field '{}' has invalid type. Expected one of: {:?}, got: {}", 
                            field, allowed, actual_type).into());
                    }
                }
            }
        } else {
            return Err("JSON must be an object".into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let mut validator = JsonValidator::new();
        validator.add_required_field("name");
        validator.add_allowed_type("age", vec!["number"]);
        
        let json = r#"{"name": "John", "age": 30}"#;
        assert!(validator.validate(json).is_ok());
    }

    #[test]
    fn test_missing_required_field() {
        let mut validator = JsonValidator::new();
        validator.add_required_field("email");
        
        let json = r#"{"name": "John"}"#;
        assert!(validator.validate(json).is_err());
    }

    #[test]
    fn test_invalid_type() {
        let mut validator = JsonValidator::new();
        validator.add_allowed_type("count", vec!["number"]);
        
        let json = r#"{"count": "five"}"#;
        assert!(validator.validate(json).is_err());
    }
}