use serde_json::{Value, from_str};
use std::collections::HashSet;
use std::error::Error;

#[derive(Debug)]
pub struct JsonValidator {
    required_fields: HashSet<String>,
    allowed_types: HashSet<&'static str>,
}

impl JsonValidator {
    pub fn new() -> Self {
        JsonValidator {
            required_fields: HashSet::new(),
            allowed_types: HashSet::from(["object", "array", "string", "number", "boolean", "null"]),
        }
    }

    pub fn add_required_field(&mut self, field: &str) {
        self.required_fields.insert(field.to_string());
    }

    pub fn validate(&self, json_str: &str) -> Result<Value, Box<dyn Error>> {
        let parsed: Value = from_str(json_str)?;
        
        if let Value::Object(ref obj) = parsed {
            for field in &self.required_fields {
                if !obj.contains_key(field) {
                    return Err(format!("Missing required field: {}", field).into());
                }
            }
        }
        
        self.validate_value_type(&parsed)?;
        Ok(parsed)
    }

    fn validate_value_type(&self, value: &Value) -> Result<(), Box<dyn Error>> {
        match value {
            Value::Object(map) => {
                for (_, v) in map {
                    self.validate_value_type(v)?;
                }
            }
            Value::Array(arr) => {
                for v in arr {
                    self.validate_value_type(v)?;
                }
            }
            _ => {
                let type_name = match value {
                    Value::Null => "null",
                    Value::Bool(_) => "boolean",
                    Value::Number(_) => "number",
                    Value::String(_) => "string",
                    _ => unreachable!(),
                };
                
                if !self.allowed_types.contains(type_name) {
                    return Err(format!("Disallowed type encountered: {}", type_name).into());
                }
            }
        }
        Ok(())
    }

    pub fn is_valid_schema(&self, json_str: &str) -> bool {
        self.validate(json_str).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let mut validator = JsonValidator::new();
        validator.add_required_field("name");
        
        let json_data = r#"{"name": "test", "value": 42}"#;
        assert!(validator.is_valid_schema(json_data));
    }

    #[test]
    fn test_missing_required_field() {
        let mut validator = JsonValidator::new();
        validator.add_required_field("id");
        
        let json_data = r#"{"name": "test"}"#;
        assert!(!validator.is_valid_schema(json_data));
    }
}