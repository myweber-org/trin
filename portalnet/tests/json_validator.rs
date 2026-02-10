use serde_json::Value;
use jsonschema::JSONSchema;

pub fn validate_json(schema: &str, data: &str) -> Result<(), Vec<String>> {
    let schema_value: Value = serde_json::from_str(schema)
        .map_err(|e| vec![format!("Invalid schema: {}", e)])?;
    
    let data_value: Value = serde_json::from_str(data)
        .map_err(|e| vec![format!("Invalid JSON data: {}", e)])?;
    
    let compiled_schema = JSONSchema::compile(&schema_value)
        .map_err(|e| vec![format!("Schema compilation failed: {}", e)])?;
    
    let validation_result = compiled_schema.validate(&data_value);
    
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let schema = r#"
        {
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "number"}
            },
            "required": ["name"]
        }
        "#;

        let valid_data = r#"{"name": "Alice", "age": 30}"#;
        assert!(validate_json(schema, valid_data).is_ok());
    }

    #[test]
    fn test_invalid_json() {
        let schema = r#"
        {
            "type": "object",
            "properties": {
                "name": {"type": "string"}
            },
            "required": ["name"]
        }
        "#;

        let invalid_data = r#"{"age": 30}"#;
        assert!(validate_json(schema, invalid_data).is_err());
    }
}use serde_json::{Value, from_str};
use std::error::Error;

pub struct JsonValidator {
    schema: Value,
}

impl JsonValidator {
    pub fn new(schema_str: &str) -> Result<Self, Box<dyn Error>> {
        let schema: Value = from_str(schema_str)?;
        Ok(JsonValidator { schema })
    }

    pub fn validate(&self, json_str: &str) -> Result<bool, Box<dyn Error>> {
        let data: Value = from_str(json_str)?;
        self.validate_value(&data)
    }

    fn validate_value(&self, data: &Value) -> Result<bool, Box<dyn Error>> {
        if let Some(required_fields) = self.schema.get("required").and_then(|v| v.as_array()) {
            if let Some(obj) = data.as_object() {
                for field in required_fields {
                    if let Some(field_str) = field.as_str() {
                        if !obj.contains_key(field_str) {
                            return Ok(false);
                        }
                    }
                }
            }
        }

        if let Some(properties) = self.schema.get("properties") {
            if let Some(obj) = data.as_object() {
                for (key, value) in obj {
                    if let Some(prop_schema) = properties.get(key) {
                        if !self.validate_type(value, prop_schema)? {
                            return Ok(false);
                        }
                    }
                }
            }
        }

        Ok(true)
    }

    fn validate_type(&self, data: &Value, schema: &Value) -> Result<bool, Box<dyn Error>> {
        if let Some(type_str) = schema.get("type").and_then(|v| v.as_str()) {
            match type_str {
                "string" => Ok(data.is_string()),
                "number" => Ok(data.is_number()),
                "integer" => Ok(data.is_i64() || data.is_u64()),
                "boolean" => Ok(data.is_boolean()),
                "array" => Ok(data.is_array()),
                "object" => Ok(data.is_object()),
                "null" => Ok(data.is_null()),
                _ => Ok(false),
            }
        } else {
            Ok(true)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_validation() {
        let schema = r#"
        {
            "type": "object",
            "required": ["name", "age"],
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "integer"}
            }
        }
        "#;

        let validator = JsonValidator::new(schema).unwrap();
        
        let valid_json = r#"{"name": "Alice", "age": 30}"#;
        assert!(validator.validate(valid_json).unwrap());
        
        let invalid_json = r#"{"name": "Bob"}"#;
        assert!(!validator.validate(invalid_json).unwrap());
        
        let type_mismatch = r#"{"name": "Charlie", "age": "thirty"}"#;
        assert!(!validator.validate(type_mismatch).unwrap());
    }
}