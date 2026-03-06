use serde_json::Value;
use jsonschema::JSONSchema;
use std::fs;

pub fn validate_json_file(file_path: &str, schema_path: &str) -> Result<(), String> {
    let json_content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read JSON file: {}", e))?;
    let schema_content = fs::read_to_string(schema_path)
        .map_err(|e| format!("Failed to read schema file: {}", e))?;

    let json_data: Value = serde_json::from_str(&json_content)
        .map_err(|e| format!("Invalid JSON: {}", e))?;
    let schema_data: Value = serde_json::from_str(&schema_content)
        .map_err(|e| format!("Invalid schema JSON: {}", e))?;

    let schema = JSONSchema::compile(&schema_data)
        .map_err(|e| format!("Schema compilation failed: {}", e))?;

    match schema.validate(&json_data) {
        Ok(_) => Ok(()),
        Err(errors) => {
            let error_messages: Vec<String> = errors.map(|e| e.to_string()).collect();
            Err(format!("Validation failed: {}", error_messages.join(", ")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_json_validation() {
        let schema = r#"{
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "number"}
            },
            "required": ["name"]
        }"#;

        let json = r#"{"name": "Alice", "age": 30}"#;

        let schema_file = NamedTempFile::new().unwrap();
        let json_file = NamedTempFile::new().unwrap();

        fs::write(schema_file.path(), schema).unwrap();
        fs::write(json_file.path(), json).unwrap();

        let result = validate_json_file(
            json_file.path().to_str().unwrap(),
            schema_file.path().to_str().unwrap()
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_json_validation() {
        let schema = r#"{
            "type": "object",
            "properties": {
                "name": {"type": "string"}
            },
            "required": ["name"]
        }"#;

        let json = r#"{"age": 30}"#;

        let schema_file = NamedTempFile::new().unwrap();
        let json_file = NamedTempFile::new().unwrap();

        fs::write(schema_file.path(), schema).unwrap();
        fs::write(json_file.path(), json).unwrap();

        let result = validate_json_file(
            json_file.path().to_str().unwrap(),
            schema_file.path().to_str().unwrap()
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Validation failed"));
    }
}