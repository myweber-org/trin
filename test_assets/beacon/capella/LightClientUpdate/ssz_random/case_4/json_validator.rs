use serde_json::{Value, from_str};
use std::fs;

pub struct JsonValidator {
    schema: Value,
}

impl JsonValidator {
    pub fn from_file(schema_path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(schema_path)
            .map_err(|e| format!("Failed to read schema file: {}", e))?;
        
        let schema: Value = from_str(&content)
            .map_err(|e| format!("Invalid JSON schema: {}", e))?;
        
        Ok(JsonValidator { schema })
    }

    pub fn validate(&self, json_str: &str) -> Result<Value, String> {
        let data: Value = from_str(json_str)
            .map_err(|e| format!("Invalid JSON data: {}", e))?;
        
        self.validate_value(&data)?;
        Ok(data)
    }

    fn validate_value(&self, data: &Value) -> Result<(), String> {
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
                ));
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_validation() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"}
            }
        });
        
        let validator = JsonValidator { schema };
        let valid_data = r#"{"name": "test"}"#;
        let invalid_data = r#"{"name": 123}"#;
        
        assert!(validator.validate(valid_data).is_ok());
        assert!(validator.validate(invalid_data).is_err());
    }
}