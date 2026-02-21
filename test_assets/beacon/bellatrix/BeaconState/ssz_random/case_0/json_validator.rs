
use serde_json::{Value, from_str};
use std::error::Error;

pub struct JsonValidator {
    schema: Value,
}

impl JsonValidator {
    pub fn new(schema_str: &str) -> Result<Self, Box<dyn Error>> {
        let schema = from_str(schema_str)?;
        Ok(JsonValidator { schema })
    }

    pub fn validate(&self, json_str: &str) -> Result<(), Box<dyn Error>> {
        let data: Value = from_str(json_str)?;
        self.validate_value(&data)
    }

    fn validate_value(&self, data: &Value) -> Result<(), Box<dyn Error>> {
        if let Some(required_type) = self.schema.get("type").and_then(|v| v.as_str()) {
            match required_type {
                "object" => self.validate_object(data)?,
                "array" => self.validate_array(data)?,
                "string" => self.validate_string(data)?,
                "number" => self.validate_number(data)?,
                "boolean" => self.validate_boolean(data)?,
                "null" => self.validate_null(data)?,
                _ => return Err("Unsupported schema type".into()),
            }
        }
        Ok(())
    }

    fn validate_object(&self, data: &Value) -> Result<(), Box<dyn Error>> {
        if !data.is_object() {
            return Err("Expected object type".into());
        }
        Ok(())
    }

    fn validate_array(&self, data: &Value) -> Result<(), Box<dyn Error>> {
        if !data.is_array() {
            return Err("Expected array type".into());
        }
        Ok(())
    }

    fn validate_string(&self, data: &Value) -> Result<(), Box<dyn Error>> {
        if !data.is_string() {
            return Err("Expected string type".into());
        }
        Ok(())
    }

    fn validate_number(&self, data: &Value) -> Result<(), Box<dyn Error>> {
        if !data.is_number() {
            return Err("Expected number type".into());
        }
        Ok(())
    }

    fn validate_boolean(&self, data: &Value) -> Result<(), Box<dyn Error>> {
        if !data.is_boolean() {
            return Err("Expected boolean type".into());
        }
        Ok(())
    }

    fn validate_null(&self, data: &Value) -> Result<(), Box<dyn Error>> {
        if !data.is_null() {
            return Err("Expected null type".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_validation() {
        let schema = r#"{"type": "string"}"#;
        let validator = JsonValidator::new(schema).unwrap();
        assert!(validator.validate(r#""hello""#).is_ok());
        assert!(validator.validate("123").is_err());
    }

    #[test]
    fn test_object_validation() {
        let schema = r#"{"type": "object"}"#;
        let validator = JsonValidator::new(schema).unwrap();
        assert!(validator.validate(r#"{"key": "value"}"#).is_ok());
        assert!(validator.validate(r#""not an object""#).is_err());
    }
}