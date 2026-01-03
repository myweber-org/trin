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
        match (&self.schema["type"].as_str(), data) {
            Some("object"), Value::Object(obj) => {
                if let Some(properties) = self.schema["properties"].as_object() {
                    for (key, prop_schema) in properties {
                        if prop_schema["required"].as_bool().unwrap_or(false) && !obj.contains_key(key) {
                            return Err(format!("Missing required field: {}", key).into());
                        }
                        if let Some(value) = obj.get(key) {
                            let validator = JsonValidator { schema: prop_schema.clone() };
                            validator.validate_value(value)?;
                        }
                    }
                }
                Ok(())
            }
            Some("array"), Value::Array(arr) => {
                if let Some(item_schema) = self.schema.get("items") {
                    for item in arr {
                        let validator = JsonValidator { schema: item_schema.clone() };
                        validator.validate_value(item)?;
                    }
                }
                Ok(())
            }
            Some("string"), Value::String(_) => Ok(()),
            Some("number"), Value::Number(_) => Ok(()),
            Some("boolean"), Value::Bool(_) => Ok(()),
            Some("null"), Value::Null => Ok(()),
            _ => Err("Type mismatch or unsupported schema".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_validation() {
        let schema = r#"
        {
            "type": "object",
            "properties": {
                "name": { "type": "string", "required": true },
                "age": { "type": "number" }
            }
        }
        "#;

        let validator = JsonValidator::new(schema).unwrap();
        let valid_json = r#"{"name": "Alice", "age": 30}"#;
        assert!(validator.validate(valid_json).is_ok());

        let invalid_json = r#"{"age": 30}"#;
        assert!(validator.validate(invalid_json).is_err());
    }
}