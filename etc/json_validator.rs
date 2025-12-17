use serde_json::{Value, Map};
use std::collections::HashSet;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ValidationError {
    field: String,
    message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Field '{}': {}", self.field, self.message)
    }
}

impl Error for ValidationError {}

pub struct JsonValidator {
    required_fields: HashSet<String>,
    field_types: Map<String, String>,
    custom_rules: Map<String, Box<dyn Fn(&Value) -> Result<(), ValidationError>>>,
}

impl JsonValidator {
    pub fn new() -> Self {
        JsonValidator {
            required_fields: HashSet::new(),
            field_types: Map::new(),
            custom_rules: Map::new(),
        }
    }

    pub fn require_field(mut self, field: &str) -> Self {
        self.required_fields.insert(field.to_string());
        self
    }

    pub fn set_field_type(mut self, field: &str, type_name: &str) -> Self {
        self.field_types.insert(field.to_string(), type_name.to_string());
        self
    }

    pub fn add_custom_rule<F>(mut self, field: &str, rule: F) -> Self
    where
        F: Fn(&Value) -> Result<(), ValidationError> + 'static,
    {
        self.custom_rules.insert(field.to_string(), Box::new(rule));
        self
    }

    pub fn validate(&self, data: &Value) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if let Value::Object(obj) = data {
            for field in &self.required_fields {
                if !obj.contains_key(field) {
                    errors.push(ValidationError {
                        field: field.clone(),
                        message: "Field is required".to_string(),
                    });
                }
            }

            for (field, value) in obj {
                if let Some(expected_type) = self.field_types.get(field) {
                    if !self.check_type(value, expected_type) {
                        errors.push(ValidationError {
                            field: field.clone(),
                            message: format!("Expected type '{}'", expected_type),
                        });
                    }
                }

                if let Some(rule) = self.custom_rules.get(field) {
                    if let Err(err) = rule(value) {
                        errors.push(err);
                    }
                }
            }
        } else {
            errors.push(ValidationError {
                field: "root".to_string(),
                message: "Expected JSON object".to_string(),
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
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

pub fn create_user_validator() -> JsonValidator {
    JsonValidator::new()
        .require_field("username")
        .require_field("email")
        .set_field_type("username", "string")
        .set_field_type("email", "string")
        .set_field_type("age", "number")
        .add_custom_rule("username", |value| {
            if let Some(s) = value.as_str() {
                if s.len() < 3 {
                    Err(ValidationError {
                        field: "username".to_string(),
                        message: "Username must be at least 3 characters".to_string(),
                    })
                } else {
                    Ok(())
                }
            } else {
                Ok(())
            }
        })
        .add_custom_rule("email", |value| {
            if let Some(email) = value.as_str() {
                if !email.contains('@') {
                    Err(ValidationError {
                        field: "email".to_string(),
                        message: "Email must contain '@' symbol".to_string(),
                    })
                } else {
                    Ok(())
                }
            } else {
                Ok(())
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_user() {
        let validator = create_user_validator();
        let data = json!({
            "username": "john_doe",
            "email": "john@example.com",
            "age": 25
        });

        assert!(validator.validate(&data).is_ok());
    }

    #[test]
    fn test_missing_required_field() {
        let validator = create_user_validator();
        let data = json!({
            "username": "john_doe"
        });

        let result = validator.validate(&data);
        assert!(result.is_err());
        if let Err(errors) = result {
            assert!(errors.iter().any(|e| e.field == "email"));
        }
    }

    #[test]
    fn test_invalid_email() {
        let validator = create_user_validator();
        let data = json!({
            "username": "john_doe",
            "email": "invalid-email",
            "age": 25
        });

        let result = validator.validate(&data);
        assert!(result.is_err());
        if let Err(errors) = result {
            assert!(errors.iter().any(|e| e.field == "email"));
        }
    }
}