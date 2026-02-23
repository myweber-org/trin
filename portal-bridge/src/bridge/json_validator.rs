use serde_json::{Value, from_str};
use std::error::Error;
use std::fs;

pub struct JsonValidator {
    schema: Value,
}

impl JsonValidator {
    pub fn from_file(schema_path: &str) -> Result<Self, Box<dyn Error>> {
        let schema_content = fs::read_to_string(schema_path)?;
        let schema: Value = from_str(&schema_content)?;
        Ok(JsonValidator { schema })
    }

    pub fn validate(&self, json_str: &str) -> Result<Value, Box<dyn Error>> {
        let data: Value = from_str(json_str)?;
        self.validate_value(&data)?;
        Ok(data)
    }

    fn validate_value(&self, data: &Value) -> Result<(), Box<dyn Error>> {
        if let Some(expected_type) = self.schema.get("type") {
            let actual_type = match data {
                Value::Null => "null",
                Value::Bool(_) => "boolean",
                Value::Number(_) => "number",
                Value::String(_) => "string",
                Value::Array(_) => "array",
                Value::Object(_) => "object",
            };

            if expected_type.as_str() != Some(actual_type) {
                return Err(format!(
                    "Type mismatch: expected {}, got {}",
                    expected_type, actual_type
                ).into());
            }
        }

        if let Some(properties) = self.schema.get("properties") {
            if let Value::Object(obj) = data {
                for (key, prop_schema) in properties.as_object().unwrap() {
                    if let Some(value) = obj.get(key) {
                        let validator = JsonValidator {
                            schema: prop_schema.clone(),
                        };
                        validator.validate_value(value)?;
                    } else if prop_schema.get("required").and_then(Value::as_bool) == Some(true) {
                        return Err(format!("Missing required property: {}", key).into());
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
    fn test_basic_validation() {
        let schema = r#"{
            "type": "object",
            "properties": {
                "name": {"type": "string", "required": true},
                "age": {"type": "number"}
            }
        }"#;

        let validator = JsonValidator::from_file("test_schema.json").unwrap();
        
        let valid_json = r#"{"name": "Alice", "age": 30}"#;
        assert!(validator.validate(valid_json).is_ok());

        let invalid_json = r#"{"name": 123}"#;
        assert!(validator.validate(invalid_json).is_err());
    }
}