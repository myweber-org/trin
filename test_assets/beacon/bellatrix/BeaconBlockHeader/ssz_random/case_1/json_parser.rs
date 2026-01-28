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
    pos: usize,
}

impl JsonParser {
    pub fn new(input: &str) -> Self {
        JsonParser {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn consume(&mut self) -> Option<char> {
        let ch = self.peek();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.consume();
            } else {
                break;
            }
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        self.parse_value()
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        match self.peek() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(ch) if ch.is_digit(10) || ch == '-' => self.parse_number(),
            _ => Err(format!("Unexpected character at position {}", self.pos)),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        let expected = "null";
        for (i, expected_ch) in expected.chars().enumerate() {
            match self.consume() {
                Some(ch) if ch == expected_ch => continue,
                _ => return Err(format!("Expected '{}' at position {}", expected, self.pos - i)),
            }
        }
        Ok(JsonValue::Null)
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        let start_pos = self.pos;
        let mut buffer = String::new();
        
        while let Some(ch) = self.peek() {
            if ch.is_alphabetic() {
                buffer.push(self.consume().unwrap());
            } else {
                break;
            }
        }
        
        match buffer.as_str() {
            "true" => Ok(JsonValue::Bool(true)),
            "false" => Ok(JsonValue::Bool(false)),
            _ => Err(format!("Invalid boolean value at position {}", start_pos)),
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.consume(); // Consume opening quote
        let mut result = String::new();
        
        while let Some(ch) = self.consume() {
            match ch {
                '"' => return Ok(JsonValue::String(result)),
                '\\' => {
                    if let Some(escaped) = self.consume() {
                        match escaped {
                            '"' => result.push('"'),
                            '\\' => result.push('\\'),
                            '/' => result.push('/'),
                            'b' => result.push('\x08'),
                            'f' => result.push('\x0c'),
                            'n' => result.push('\n'),
                            'r' => result.push('\r'),
                            't' => result.push('\t'),
                            _ => return Err(format!("Invalid escape sequence at position {}", self.pos - 2)),
                        }
                    } else {
                        return Err("Unterminated string".to_string());
                    }
                }
                _ => result.push(ch),
            }
        }
        
        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start_pos = self.pos;
        let mut buffer = String::new();
        
        if let Some('-') = self.peek() {
            buffer.push(self.consume().unwrap());
        }
        
        while let Some(ch) = self.peek() {
            if ch.is_digit(10) {
                buffer.push(self.consume().unwrap());
            } else {
                break;
            }
        }
        
        if let Some('.') = self.peek() {
            buffer.push(self.consume().unwrap());
            while let Some(ch) = self.peek() {
                if ch.is_digit(10) {
                    buffer.push(self.consume().unwrap());
                } else {
                    break;
                }
            }
        }
        
        match buffer.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(format!("Invalid number at position {}", start_pos)),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.consume(); // Consume '['
        let mut array = Vec::new();
        
        self.skip_whitespace();
        
        if let Some(']') = self.peek() {
            self.consume();
            return Ok(JsonValue::Array(array));
        }
        
        loop {
            self.skip_whitespace();
            let value = self.parse_value()?;
            array.push(value);
            
            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.consume();
                    continue;
                }
                Some(']') => {
                    self.consume();
                    break;
                }
                _ => return Err(format!("Expected ',' or ']' at position {}", self.pos)),
            }
        }
        
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.consume(); // Consume '{'
        let mut object = HashMap::new();
        
        self.skip_whitespace();
        
        if let Some('}') = self.peek() {
            self.consume();
            return Ok(JsonValue::Object(object));
        }
        
        loop {
            self.skip_whitespace();
            
            if let Some('"') = self.peek() {
                let key = match self.parse_string()? {
                    JsonValue::String(s) => s,
                    _ => return Err("Expected string key".to_string()),
                };
                
                self.skip_whitespace();
                match self.consume() {
                    Some(':') => (),
                    _ => return Err("Expected ':' after object key".to_string()),
                }
                
                self.skip_whitespace();
                let value = self.parse_value()?;
                object.insert(key, value);
                
                self.skip_whitespace();
                match self.peek() {
                    Some(',') => {
                        self.consume();
                        continue;
                    }
                    Some('}') => {
                        self.consume();
                        break;
                    }
                    _ => return Err("Expected ',' or '}' after object value".to_string()),
                }
            } else {
                return Err("Expected string key in object".to_string());
            }
        }
        
        Ok(JsonValue::Object(object))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_null() {
        let mut parser = JsonParser::new("null");
        assert_eq!(parser.parse(), Ok(JsonValue::Null));
    }

    #[test]
    fn test_parse_bool() {
        let mut parser = JsonParser::new("true");
        assert_eq!(parser.parse(), Ok(JsonValue::Bool(true)));
        
        let mut parser = JsonParser::new("false");
        assert_eq!(parser.parse(), Ok(JsonValue::Bool(false)));
    }

    #[test]
    fn test_parse_string() {
        let mut parser = JsonParser::new(r#""hello world""#);
        assert_eq!(parser.parse(), Ok(JsonValue::String("hello world".to_string())));
    }

    #[test]
    fn test_parse_number() {
        let mut parser = JsonParser::new("42");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(42.0)));
        
        let mut parser = JsonParser::new("-3.14");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(-3.14)));
    }

    #[test]
    fn test_parse_array() {
        let mut parser = JsonParser::new("[1, 2, 3]");
        let expected = JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::Number(2.0),
            JsonValue::Number(3.0),
        ]);
        assert_eq!(parser.parse(), Ok(expected));
    }

    #[test]
    fn test_parse_object() {
        let mut parser = JsonParser::new(r#"{"key": "value"}"#);
        let mut expected_map = HashMap::new();
        expected_map.insert("key".to_string(), JsonValue::String("value".to_string()));
        let expected = JsonValue::Object(expected_map);
        assert_eq!(parser.parse(), Ok(expected));
    }
}