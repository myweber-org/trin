use serde_json::{Value, Map};
use std::collections::HashSet;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error at {}: {}", self.path, self.message)
    }
}

impl Error for ValidationError {}

pub struct JsonValidator {
    required_fields: HashSet<String>,
    field_types: Map<String, String>,
    allow_extra_fields: bool,
}

impl JsonValidator {
    pub fn new() -> Self {
        JsonValidator {
            required_fields: HashSet::new(),
            field_types: Map::new(),
            allow_extra_fields: false,
        }
    }

    pub fn require_field(mut self, field: &str) -> Self {
        self.required_fields.insert(field.to_string());
        self
    }

    pub fn define_type(mut self, field: &str, type_name: &str) -> Self {
        self.field_types.insert(field.to_string(), type_name.to_string());
        self
    }

    pub fn allow_extra_fields(mut self, allow: bool) -> Self {
        self.allow_extra_fields = allow;
        self
    }

    pub fn validate(&self, data: &Value) -> Result<(), ValidationError> {
        let obj = match data.as_object() {
            Some(o) => o,
            None => return Err(ValidationError {
                path: "root".to_string(),
                message: "Expected JSON object".to_string(),
            }),
        };

        for field in &self.required_fields {
            if !obj.contains_key(field) {
                return Err(ValidationError {
                    path: field.clone(),
                    message: "Required field is missing".to_string(),
                });
            }
        }

        for (field, value) in obj {
            if let Some(expected_type) = self.field_types.get(field) {
                if !self.check_type(value, expected_type) {
                    return Err(ValidationError {
                        path: field.clone(),
                        message: format!("Expected type '{}'", expected_type),
                    });
                }
            } else if !self.allow_extra_fields {
                return Err(ValidationError {
                    path: field.clone(),
                    message: "Unexpected field".to_string(),
                });
            }
        }

        Ok(())
    }

    fn check_type(&self, value: &Value, expected_type: &str) -> bool {
        match expected_type {
            "string" => value.is_string(),
            "number" => value.is_number(),
            "boolean" => value.is_boolean(),
            "array" => value.is_array(),
            "object" => value.is_object(),
            "null" => value.is_null(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validation_success() {
        let validator = JsonValidator::new()
            .require_field("name")
            .define_type("name", "string")
            .define_type("age", "number")
            .allow_extra_fields(true);

        let data = json!({
            "name": "John",
            "age": 30,
            "extra": "field"
        });

        assert!(validator.validate(&data).is_ok());
    }

    #[test]
    fn test_missing_required_field() {
        let validator = JsonValidator::new()
            .require_field("name")
            .define_type("name", "string");

        let data = json!({
            "age": 30
        });

        assert!(validator.validate(&data).is_err());
    }

    #[test]
    fn test_type_mismatch() {
        let validator = JsonValidator::new()
            .define_type("age", "number");

        let data = json!({
            "age": "thirty"
        });

        assert!(validator.validate(&data).is_err());
    }
}