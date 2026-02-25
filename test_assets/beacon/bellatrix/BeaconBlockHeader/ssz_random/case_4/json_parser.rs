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

pub struct JsonParser {
    input: Vec<char>,
    position: usize,
}

impl JsonParser {
    pub fn new(input: &str) -> Self {
        JsonParser {
            input: input.chars().collect(),
            position: 0,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && self.input[self.position].is_whitespace() {
            self.position += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        if self.position < self.input.len() {
            Some(self.input[self.position])
        } else {
            None
        }
    }

    fn consume(&mut self, expected: char) -> Result<(), String> {
        self.skip_whitespace();
        if self.peek() == Some(expected) {
            self.position += 1;
            Ok(())
        } else {
            Err(format!("Expected '{}', found {:?}", expected, self.peek()))
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.consume('"')?;
        let mut result = String::new();
        
        while self.position < self.input.len() {
            let ch = self.input[self.position];
            self.position += 1;
            
            match ch {
                '"' => return Ok(result),
                '\\' => {
                    if self.position >= self.input.len() {
                        return Err("Unterminated escape sequence".to_string());
                    }
                    let escaped = self.input[self.position];
                    self.position += 1;
                    match escaped {
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        '/' => result.push('/'),
                        'b' => result.push('\x08'),
                        'f' => result.push('\x0c'),
                        'n' => result.push('\n'),
                        'r' => result.push('\r'),
                        't' => result.push('\t'),
                        _ => return Err(format!("Invalid escape sequence: \\{}", escaped)),
                    }
                }
                _ => result.push(ch),
            }
        }
        
        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<f64, String> {
        let start = self.position;
        let mut has_dot = false;
        
        while self.position < self.input.len() {
            let ch = self.input[self.position];
            if ch.is_ascii_digit() {
                self.position += 1;
            } else if ch == '.' && !has_dot {
                has_dot = true;
                self.position += 1;
            } else {
                break;
            }
        }
        
        if self.position == start {
            return Err("Expected number".to_string());
        }
        
        let num_str: String = self.input[start..self.position].iter().collect();
        num_str.parse().map_err(|e| format!("Invalid number: {}", e))
    }

    fn parse_array(&mut self) -> Result<Vec<JsonValue>, String> {
        self.consume('[')?;
        self.skip_whitespace();
        
        if self.peek() == Some(']') {
            self.position += 1;
            return Ok(Vec::new());
        }
        
        let mut array = Vec::new();
        loop {
            let value = self.parse_value()?;
            array.push(value);
            
            self.skip_whitespace();
            if self.peek() == Some(']') {
                self.position += 1;
                break;
            }
            
            self.consume(',')?;
        }
        
        Ok(array)
    }

    fn parse_object(&mut self) -> Result<HashMap<String, JsonValue>, String> {
        self.consume('{')?;
        self.skip_whitespace();
        
        if self.peek() == Some('}') {
            self.position += 1;
            return Ok(HashMap::new());
        }
        
        let mut map = HashMap::new();
        loop {
            let key = self.parse_string()?;
            self.consume(':')?;
            let value = self.parse_value()?;
            map.insert(key, value);
            
            self.skip_whitespace();
            if self.peek() == Some('}') {
                self.position += 1;
                break;
            }
            
            self.consume(',')?;
        }
        
        Ok(map)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        
        match self.peek() {
            Some('n') => {
                if self.position + 3 < self.input.len() 
                    && self.input[self.position..self.position + 4].iter().collect::<String>() == "null" {
                    self.position += 4;
                    Ok(JsonValue::Null)
                } else {
                    Err("Expected 'null'".to_string())
                }
            }
            Some('t') => {
                if self.position + 3 < self.input.len() 
                    && self.input[self.position..self.position + 4].iter().collect::<String>() == "true" {
                    self.position += 4;
                    Ok(JsonValue::Bool(true))
                } else {
                    Err("Expected 'true'".to_string())
                }
            }
            Some('f') => {
                if self.position + 4 < self.input.len() 
                    && self.input[self.position..self.position + 5].iter().collect::<String>() == "false" {
                    self.position += 5;
                    Ok(JsonValue::Bool(false))
                } else {
                    Err("Expected 'false'".to_string())
                }
            }
            Some('"') => {
                let s = self.parse_string()?;
                Ok(JsonValue::String(s))
            }
            Some('[') => {
                let arr = self.parse_array()?;
                Ok(JsonValue::Array(arr))
            }
            Some('{') => {
                let obj = self.parse_object()?;
                Ok(JsonValue::Object(obj))
            }
            Some(ch) if ch.is_ascii_digit() || ch == '-' => {
                let num = self.parse_number()?;
                Ok(JsonValue::Number(num))
            }
            _ => Err(format!("Unexpected character: {:?}", self.peek())),
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.position < self.input.len() {
            Err("Trailing characters after JSON value".to_string())
        } else {
            Ok(result)
        }
    }
}

pub fn parse_json(json_str: &str) -> Result<JsonValue, String> {
    let mut parser = JsonParser::new(json_str);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_json() {
        let json = r#"{"name": "test", "value": 42.5, "active": true}"#;
        let result = parse_json(json);
        assert!(result.is_ok());
        
        if let Ok(JsonValue::Object(map)) = result {
            assert_eq!(map.get("name"), Some(&JsonValue::String("test".to_string())));
            assert_eq!(map.get("value"), Some(&JsonValue::Number(42.5)));
            assert_eq!(map.get("active"), Some(&JsonValue::Bool(true)));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_parse_array() {
        let json = r#"[1, 2, 3, "four", false]"#;
        let result = parse_json(json);
        assert!(result.is_ok());
        
        if let Ok(JsonValue::Array(arr)) = result {
            assert_eq!(arr.len(), 5);
            assert_eq!(arr[0], JsonValue::Number(1.0));
            assert_eq!(arr[3], JsonValue::String("four".to_string()));
            assert_eq!(arr[4], JsonValue::Bool(false));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_parse_nested() {
        let json = r#"{"data": {"items": [1, 2], "metadata": null}}"#;
        let result = parse_json(json);
        assert!(result.is_ok());
    }
}