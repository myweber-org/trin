use serde_json::Value;
use jsonschema::JSONSchema;

pub fn validate_json(schema: &str, data: &str) -> Result<(), Vec<String>> {
    let schema_value: Value = serde_json::from_str(schema).unwrap();
    let data_value: Value = serde_json::from_str(data).unwrap();
    
    let compiled = JSONSchema::compile(&schema_value).unwrap();
    let validation_result = compiled.validate(&data_value);
    
    match validation_result {
        Ok(_) => Ok(()),
        Err(errors) => {
            let error_messages: Vec<String> = errors
                .map(|e| e.to_string())
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

        let data = r#"{"name": "John", "age": 30}"#;
        
        assert!(validate_json(schema, data).is_ok());
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

        let data = r#"{"age": 30}"#;
        
        assert!(validate_json(schema, data).is_err());
    }
}