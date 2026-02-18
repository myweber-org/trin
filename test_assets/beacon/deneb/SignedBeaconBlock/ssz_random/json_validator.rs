use serde_json::{Value, Map};
use std::collections::HashSet;

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

    pub fn validate(&self, json_str: &str) -> Result<(), String> {
        let parsed: Value = serde_json::from_str(json_str)
            .map_err(|e| format!("Invalid JSON: {}", e))?;

        self.validate_value(&parsed)
    }

    fn validate_value(&self, value: &Value) -> Result<(), String> {
        if let Value::Object(obj) = value {
            for field in &self.required_fields {
                if !obj.contains_key(field) {
                    return Err(format!("Missing required field: {}", field));
                }
            }

            for (key, val) in obj {
                if let Some(allowed) = self.allowed_types.get(key) {
                    let current_type = match val {
                        Value::Null => "null",
                        Value::Bool(_) => "boolean",
                        Value::Number(_) => "number",
                        Value::String(_) => "string",
                        Value::Array(_) => "array",
                        Value::Object(_) => "object",
                    };

                    if !allowed.contains(&current_type.to_string()) {
                        return Err(format!(
                            "Field '{}' has invalid type '{}'. Allowed: {:?}",
                            key, current_type, allowed
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let mut validator = JsonValidator::new();
        validator.add_required_field("name");
        validator.add_allowed_type("age", vec!["number"]);
        validator.add_allowed_type("tags", vec!["array"]);

        let valid_json = r#"{"name": "John", "age": 30, "tags": ["rust", "json"]}"#;
        assert!(validator.validate(valid_json).is_ok());

        let missing_field = r#"{"age": 30}"#;
        assert!(validator.validate(missing_field).is_err());

        let wrong_type = r#"{"name": "John", "age": "thirty"}"#;
        assert!(validator.validate(wrong_type).is_err());
    }
}use serde_json::Value;
use jsonschema::JSONSchema;

pub fn validate_json(schema: &Value, data: &Value) -> Result<(), Vec<String>> {
    let compiled = JSONSchema::compile(schema)
        .map_err(|e| vec![format!("Schema compilation failed: {}", e)])?;
    
    let validation_result = compiled.validate(data);
    match validation_result {
        Ok(_) => Ok(()),
        Err(errors) => {
            let error_messages: Vec<String> = errors
                .map(|e| format!("Validation error: {}", e))
                .collect();
            Err(error_messages)
        }
    }
}

pub fn create_person_schema() -> Value {
    serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": {
            "name": {
                "type": "string",
                "minLength": 1
            },
            "age": {
                "type": "integer",
                "minimum": 0
            },
            "email": {
                "type": "string",
                "format": "email"
            }
        },
        "required": ["name", "age"],
        "additionalProperties": false
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_person() {
        let schema = create_person_schema();
        let data = serde_json::json!({
            "name": "John Doe",
            "age": 30,
            "email": "john@example.com"
        });
        
        assert!(validate_json(&schema, &data).is_ok());
    }

    #[test]
    fn test_invalid_person_missing_required() {
        let schema = create_person_schema();
        let data = serde_json::json!({
            "name": "John Doe"
        });
        
        assert!(validate_json(&schema, &data).is_err());
    }

    #[test]
    fn test_invalid_person_wrong_type() {
        let schema = create_person_schema();
        let data = serde_json::json!({
            "name": "John Doe",
            "age": "thirty"
        });
        
        assert!(validate_json(&schema, &data).is_err());
    }
}