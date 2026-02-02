use serde_json::{Value, from_str};
use std::error::Error;

pub fn validate_json_schema(json_str: &str, schema: &Value) -> Result<(), Box<dyn Error>> {
    let data: Value = from_str(json_str)?;
    
    if let Value::Object(schema_map) = schema {
        if let Value::Object(data_map) = &data {
            for (key, schema_value) in schema_map {
                match data_map.get(key) {
                    Some(data_value) => {
                        if !validate_type(data_value, schema_value) {
                            return Err(format!("Field '{}' type mismatch", key).into());
                        }
                    }
                    None => {
                        if !is_optional(schema_value) {
                            return Err(format!("Required field '{}' missing", key).into());
                        }
                    }
                }
            }
        } else {
            return Err("Root must be JSON object".into());
        }
    } else {
        return Err("Schema must be JSON object".into());
    }
    
    Ok(())
}

fn validate_type(data: &Value, schema: &Value) -> bool {
    match schema {
        Value::String(type_str) => match type_str.as_str() {
            "string" => data.is_string(),
            "number" => data.is_number(),
            "boolean" => data.is_boolean(),
            "object" => data.is_object(),
            "array" => data.is_array(),
            "null" => data.is_null(),
            _ => false,
        },
        Value::Object(obj) if obj.contains_key("type") => {
            if let Some(Value::String(type_str)) = obj.get("type") {
                validate_type(data, &Value::String(type_str.clone()))
            } else {
                false
            }
        }
        _ => false,
    }
}

fn is_optional(schema: &Value) -> bool {
    if let Value::Object(obj) = schema {
        if let Some(Value::Bool(optional)) = obj.get("optional") {
            *optional
        } else {
            false
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_json() {
        let schema = json!({
            "name": "string",
            "age": "number",
            "active": "boolean"
        });
        
        let data = r#"{"name": "Alice", "age": 30, "active": true}"#;
        
        assert!(validate_json_schema(data, &schema).is_ok());
    }

    #[test]
    fn test_missing_required_field() {
        let schema = json!({
            "name": "string",
            "age": "number"
        });
        
        let data = r#"{"name": "Bob"}"#;
        
        assert!(validate_json_schema(data, &schema).is_err());
    }
}