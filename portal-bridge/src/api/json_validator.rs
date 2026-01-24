use serde_json::Value;
use jsonschema::JSONSchema;

pub fn validate_json(schema: &Value, data: &Value) -> Result<(), Vec<String>> {
    let compiled_schema = JSONSchema::compile(schema)
        .map_err(|e| vec![format!("Schema compilation failed: {}", e)])?;
    
    let validation_result = compiled_schema.validate(data);
    
    if validation_result.is_ok() {
        Ok(())
    } else {
        let errors: Vec<String> = validation_result
            .unwrap_err()
            .map(|error| format!("Validation error: {}", error))
            .collect();
        Err(errors)
    }
}

pub fn load_schema_from_str(schema_str: &str) -> Result<Value, String> {
    serde_json::from_str(schema_str)
        .map_err(|e| format!("Failed to parse schema: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_json_validation() {
        let schema = r#"
        {
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "number", "minimum": 0}
            },
            "required": ["name"]
        }
        "#;
        
        let data = r#"
        {
            "name": "John Doe",
            "age": 30
        }
        "#;
        
        let schema_value = load_schema_from_str(schema).unwrap();
        let data_value = serde_json::from_str(data).unwrap();
        
        assert!(validate_json(&schema_value, &data_value).is_ok());
    }
    
    #[test]
    fn test_invalid_json_validation() {
        let schema = r#"
        {
            "type": "object",
            "properties": {
                "email": {"type": "string", "format": "email"}
            },
            "required": ["email"]
        }
        "#;
        
        let invalid_data = r#"
        {
            "email": "not-an-email"
        }
        "#;
        
        let schema_value = load_schema_from_str(schema).unwrap();
        let data_value = serde_json::from_str(invalid_data).unwrap();
        
        let result = validate_json(&schema_value, &data_value);
        assert!(result.is_err());
    }
}