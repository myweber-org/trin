use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

pub fn parse_json(input: &str) -> Result<JsonValue, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Empty input".to_string());
    }

    match trimmed.chars().next().unwrap() {
        'n' => parse_null(trimmed),
        't' | 'f' => parse_bool(trimmed),
        '"' => parse_string(trimmed),
        '[' => parse_array(trimmed),
        '{' => parse_object(trimmed),
        '-' | '0'..='9' => parse_number(trimmed),
        _ => Err("Invalid JSON".to_string()),
    }
}

fn parse_null(input: &str) -> Result<JsonValue, String> {
    if input == "null" {
        Ok(JsonValue::Null)
    } else {
        Err("Expected null".to_string())
    }
}

fn parse_bool(input: &str) -> Result<JsonValue, String> {
    if input == "true" {
        Ok(JsonValue::Bool(true))
    } else if input == "false" {
        Ok(JsonValue::Bool(false))
    } else {
        Err("Expected boolean".to_string())
    }
}

fn parse_string(input: &str) -> Result<JsonValue, String> {
    if input.starts_with('"') && input.ends_with('"') {
        let content = &input[1..input.len() - 1];
        Ok(JsonValue::String(content.to_string()))
    } else {
        Err("Invalid string".to_string())
    }
}

fn parse_number(input: &str) -> Result<JsonValue, String> {
    match input.parse::<f64>() {
        Ok(num) => Ok(JsonValue::Number(num)),
        Err(_) => Err("Invalid number".to_string()),
    }
}

fn parse_array(input: &str) -> Result<JsonValue, String> {
    if !input.starts_with('[') || !input.ends_with(']') {
        return Err("Invalid array".to_string());
    }

    let content = &input[1..input.len() - 1].trim();
    if content.is_empty() {
        return Ok(JsonValue::Array(Vec::new()));
    }

    let mut elements = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    let mut in_string = false;

    for ch in content.chars() {
        match ch {
            '"' if !current.ends_with('\\') => in_string = !in_string,
            '[' | '{' if !in_string => depth += 1,
            ']' | '}' if !in_string => depth -= 1,
            ',' if depth == 0 && !in_string => {
                if !current.trim().is_empty() {
                    elements.push(parse_json(&current.trim())?);
                }
                current.clear();
                continue;
            }
            _ => {}
        }
        current.push(ch);
    }

    if !current.trim().is_empty() {
        elements.push(parse_json(&current.trim())?);
    }

    Ok(JsonValue::Array(elements))
}

fn parse_object(input: &str) -> Result<JsonValue, String> {
    if !input.starts_with('{') || !input.ends_with('}') {
        return Err("Invalid object".to_string());
    }

    let content = &input[1..input.len() - 1].trim();
    if content.is_empty() {
        return Ok(JsonValue::Object(HashMap::new()));
    }

    let mut map = HashMap::new();
    let mut current_key = String::new();
    let mut current_value = String::new();
    let mut parsing_key = true;
    let mut depth = 0;
    let mut in_string = false;

    for ch in content.chars() {
        match ch {
            '"' if !current_value.ends_with('\\') => in_string = !in_string,
            '[' | '{' if !in_string => depth += 1,
            ']' | '}' if !in_string => depth -= 1,
            ':' if depth == 0 && !in_string && parsing_key => {
                parsing_key = false;
                current_key = current_value.trim().to_string();
                if !current_key.starts_with('"') || !current_key.ends_with('"') {
                    return Err("Object key must be string".to_string());
                }
                current_key = current_key[1..current_key.len() - 1].to_string();
                current_value.clear();
                continue;
            }
            ',' if depth == 0 && !in_string && !parsing_key => {
                if !current_key.is_empty() && !current_value.trim().is_empty() {
                    let value = parse_json(&current_value.trim())?;
                    map.insert(current_key.clone(), value);
                }
                current_key.clear();
                current_value.clear();
                parsing_key = true;
                continue;
            }
            _ => {}
        }
        current_value.push(ch);
    }

    if !current_key.is_empty() && !current_value.trim().is_empty() {
        let value = parse_json(&current_value.trim())?;
        map.insert(current_key, value);
    }

    Ok(JsonValue::Object(map))
}