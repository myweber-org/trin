use serde_json::Value;
use jsonschema::JSONSchema;

pub fn validate_json(schema: &Value, data: &Value) -> Result<(), Vec<String>> {
    let compiled = JSONSchema::compile(schema)
        .map_err(|e| vec![format!("Schema compilation failed: {}", e)])?;
    
    let validation_result = compiled.validate(data);
    match validation_result {
        Ok(_) => Ok(()),
        Err(errors) => {
            let error_messages: Vec<String> = errors
                .map(|error| format!("Validation error: {}", error))
                .collect();
            Err(error_messages)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_json() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "number", "minimum": 0}
            },
            "required": ["name"]
        });

        let data = json!({
            "name": "John",
            "age": 30
        });

        assert!(validate_json(&schema, &data).is_ok());
    }

    #[test]
    fn test_invalid_json() {
        let schema = json!({
            "type": "object",
            "properties": {
                "email": {"type": "string", "format": "email"}
            },
            "required": ["email"]
        });

        let data = json!({
            "email": "not-an-email"
        });

        assert!(validate_json(&schema, &data).is_err());
    }
}