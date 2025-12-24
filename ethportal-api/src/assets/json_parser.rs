use std::collections::HashMap;

#[derive(Debug, PartialEq)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

struct JsonParser {
    input: String,
    pos: usize,
}

impl JsonParser {
    fn new(input: &str) -> Self {
        JsonParser {
            input: input.to_string(),
            pos: 0,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            if c.is_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        if self.pos >= self.input.len() {
            return Err("Unexpected end of input".to_string());
        }

        let c = self.input.chars().nth(self.pos).unwrap();
        match c {
            'n' => self.parse_null(),
            't' | 'f' => self.parse_bool(),
            '"' => self.parse_string(),
            '[' => self.parse_array(),
            '{' => self.parse_object(),
            '-' | '0'..='9' => self.parse_number(),
            _ => Err(format!("Unexpected character: {}", c)),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        if self.input[self.pos..].starts_with("null") {
            self.pos += 4;
            Ok(JsonValue::Null)
        } else {
            Err("Expected 'null'".to_string())
        }
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.input[self.pos..].starts_with("true") {
            self.pos += 4;
            Ok(JsonValue::Bool(true))
        } else if self.input[self.pos..].starts_with("false") {
            self.pos += 5;
            Ok(JsonValue::Bool(false))
        } else {
            Err("Expected 'true' or 'false'".to_string())
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.pos += 1; // Skip opening quote
        let start = self.pos;
        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            if c == '"' {
                let s = self.input[start..self.pos].to_string();
                self.pos += 1;
                return Ok(JsonValue::String(s));
            }
            self.pos += 1;
        }
        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            if !c.is_digit(10) && c != '.' && c != '-' && c != 'e' && c != 'E' && c != '+' {
                break;
            }
            self.pos += 1;
        }
        let num_str = &self.input[start..self.pos];
        match num_str.parse::<f64>() {
            Ok(n) => Ok(JsonValue::Number(n)),
            Err(_) => Err(format!("Invalid number: {}", num_str)),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.pos += 1; // Skip '['
        let mut array = Vec::new();

        self.skip_whitespace();
        if self.pos < self.input.len() && self.input.chars().nth(self.pos).unwrap() == ']' {
            self.pos += 1;
            return Ok(JsonValue::Array(array));
        }

        loop {
            let value = self.parse_value()?;
            array.push(value);

            self.skip_whitespace();
            if self.pos >= self.input.len() {
                return Err("Unterminated array".to_string());
            }

            let c = self.input.chars().nth(self.pos).unwrap();
            if c == ']' {
                self.pos += 1;
                break;
            } else if c == ',' {
                self.pos += 1;
                self.skip_whitespace();
            } else {
                return Err(format!("Expected ',' or ']', found: {}", c));
            }
        }

        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.pos += 1; // Skip '{'
        let mut object = HashMap::new();

        self.skip_whitespace();
        if self.pos < self.input.len() && self.input.chars().nth(self.pos).unwrap() == '}' {
            self.pos += 1;
            return Ok(JsonValue::Object(object));
        }

        loop {
            let key = match self.parse_value()? {
                JsonValue::String(s) => s,
                _ => return Err("Object key must be a string".to_string()),
            };

            self.skip_whitespace();
            if self.pos >= self.input.len() || self.input.chars().nth(self.pos).unwrap() != ':' {
                return Err("Expected ':' after object key".to_string());
            }
            self.pos += 1;

            let value = self.parse_value()?;
            object.insert(key, value);

            self.skip_whitespace();
            if self.pos >= self.input.len() {
                return Err("Unterminated object".to_string());
            }

            let c = self.input.chars().nth(self.pos).unwrap();
            if c == '}' {
                self.pos += 1;
                break;
            } else if c == ',' {
                self.pos += 1;
                self.skip_whitespace();
            } else {
                return Err(format!("Expected ',' or '}}', found: {}", c));
            }
        }

        Ok(JsonValue::Object(object))
    }

    fn parse(&mut self) -> Result<JsonValue, String> {
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            return Err("Trailing characters after JSON value".to_string());
        }
        Ok(result)
    }
}

fn extract_key_values(json: &JsonValue, prefix: &str, result: &mut HashMap<String, String>) {
    match json {
        JsonValue::Object(obj) => {
            for (key, value) in obj {
                let new_prefix = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                extract_key_values(value, &new_prefix, result);
            }
        }
        JsonValue::String(s) => {
            result.insert(prefix.to_string(), s.clone());
        }
        JsonValue::Number(n) => {
            result.insert(prefix.to_string(), n.to_string());
        }
        JsonValue::Bool(b) => {
            result.insert(prefix.to_string(), b.to_string());
        }
        JsonValue::Null => {
            result.insert(prefix.to_string(), "null".to_string());
        }
        JsonValue::Array(arr) => {
            for (i, value) in arr.iter().enumerate() {
                let new_prefix = format!("{}[{}]", prefix, i);
                extract_key_values(value, &new_prefix, result);
            }
        }
    }
}

pub fn parse_json(input: &str) -> Result<HashMap<String, String>, String> {
    let mut parser = JsonParser::new(input);
    let json_value = parser.parse()?;
    let mut result = HashMap::new();
    extract_key_values(&json_value, "", &mut result);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_object() {
        let input = r#"{"name": "John", "age": 30, "active": true}"#;
        let result = parse_json(input).unwrap();
        assert_eq!(result.get("name"), Some(&"John".to_string()));
        assert_eq!(result.get("age"), Some(&"30".to_string()));
        assert_eq!(result.get("active"), Some(&"true".to_string()));
    }

    #[test]
    fn test_nested_object() {
        let input = r#"{"user": {"name": "Alice", "settings": {"theme": "dark"}}}"#;
        let result = parse_json(input).unwrap();
        assert_eq!(result.get("user.name"), Some(&"Alice".to_string()));
        assert_eq!(result.get("user.settings.theme"), Some(&"dark".to_string()));
    }
}