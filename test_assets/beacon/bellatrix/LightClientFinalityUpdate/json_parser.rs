use serde_json::{Value, Error as JsonError};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ParseError {
    InvalidJson(JsonError),
    MissingField(String),
    TypeMismatch(String),
    ValidationFailed(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidJson(e) => write!(f, "Invalid JSON: {}", e),
            ParseError::MissingField(field) => write!(f, "Missing required field: {}", field),
            ParseError::TypeMismatch(field) => write!(f, "Type mismatch for field: {}", field),
            ParseError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for ParseError {}

impl From<JsonError> for ParseError {
    fn from(error: JsonError) -> Self {
        ParseError::InvalidJson(error)
    }
}

pub struct JsonParser {
    schema: HashMap<String, FieldType>,
}

#[derive(Debug, Clone)]
pub enum FieldType {
    String,
    Number,
    Boolean,
    Object,
    Array,
    Custom(Box<dyn Fn(&Value) -> bool>),
}

impl JsonParser {
    pub fn new() -> Self {
        JsonParser {
            schema: HashMap::new(),
        }
    }

    pub fn add_field(&mut self, name: &str, field_type: FieldType) {
        self.schema.insert(name.to_string(), field_type);
    }

    pub fn parse(&self, json_str: &str) -> Result<Value, ParseError> {
        let value: Value = serde_json::from_str(json_str)?;

        self.validate(&value)?;

        Ok(value)
    }

    fn validate(&self, value: &Value) -> Result<(), ParseError> {
        let obj = value.as_object().ok_or_else(|| {
            ParseError::ValidationFailed("Expected JSON object".to_string())
        })?;

        for (field_name, field_type) in &self.schema {
            let field_value = obj.get(field_name)
                .ok_or_else(|| ParseError::MissingField(field_name.clone()))?;

            if !self.check_type(field_value, field_type) {
                return Err(ParseError::TypeMismatch(field_name.clone()));
            }
        }

        Ok(())
    }

    fn check_type(&self, value: &Value, field_type: &FieldType) -> bool {
        match field_type {
            FieldType::String => value.is_string(),
            FieldType::Number => value.is_number(),
            FieldType::Boolean => value.is_boolean(),
            FieldType::Object => value.is_object(),
            FieldType::Array => value.is_array(),
            FieldType::Custom(validator) => validator(value),
        }
    }

    pub fn extract_string(&self, value: &Value, field: &str) -> Result<String, ParseError> {
        value.get(field)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ParseError::MissingField(field.to_string()))
    }

    pub fn extract_number(&self, value: &Value, field: &str) -> Result<f64, ParseError> {
        value.get(field)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| ParseError::MissingField(field.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json_parsing() {
        let mut parser = JsonParser::new();
        parser.add_field("name", FieldType::String);
        parser.add_field("age", FieldType::Number);

        let json_data = r#"{"name": "John", "age": 30}"#;
        let result = parser.parse(json_data);

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(parser.extract_string(&value, "name").unwrap(), "John");
        assert_eq!(parser.extract_number(&value, "age").unwrap(), 30.0);
    }

    #[test]
    fn test_missing_field() {
        let mut parser = JsonParser::new();
        parser.add_field("name", FieldType::String);

        let json_data = r#"{"age": 30}"#;
        let result = parser.parse(json_data);

        assert!(matches!(result, Err(ParseError::MissingField(_))));
    }

    #[test]
    fn test_type_mismatch() {
        let mut parser = JsonParser::new();
        parser.add_field("age", FieldType::Number);

        let json_data = r#"{"age": "thirty"}"#;
        let result = parser.parse(json_data);

        assert!(matches!(result, Err(ParseError::TypeMismatch(_))));
    }
}