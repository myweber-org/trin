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
        Self {
            input: input.chars().collect(),
            position: 0,
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        let result = self.parse_value()?;
        self.skip_whitespace();
        
        if self.position < self.input.len() {
            return Err(format!("Unexpected character at position {}", self.position));
        }
        
        Ok(result)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        
        match self.current_char() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(c) if c.is_digit(10) || c == '-' => self.parse_number(),
            _ => Err(format!("Invalid JSON value at position {}", self.position)),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        if self.consume_str("null") {
            Ok(JsonValue::Null)
        } else {
            Err("Expected 'null'".to_string())
        }
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.consume_str("true") {
            Ok(JsonValue::Bool(true))
        } else if self.consume_str("false") {
            Ok(JsonValue::Bool(false))
        } else {
            Err("Expected boolean value".to_string())
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.consume_char('"');
        let mut result = String::new();
        
        while let Some(c) = self.current_char() {
            if c == '"' {
                break;
            }
            if c == '\\' {
                self.advance();
                if let Some(escaped) = self.current_char() {
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
                    self.advance();
                }
            } else {
                result.push(c);
                self.advance();
            }
        }
        
        if self.consume_char('"') {
            Ok(JsonValue::String(result))
        } else {
            Err("Unterminated string".to_string())
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.position;
        let mut has_decimal = false;
        let mut has_exponent = false;
        
        if self.current_char() == Some('-') {
            self.advance();
        }
        
        if self.current_char() == Some('0') {
            self.advance();
        } else if let Some(c) = self.current_char() {
            if c.is_digit(10) && c != '0' {
                while let Some(c) = self.current_char() {
                    if !c.is_digit(10) {
                        break;
                    }
                    self.advance();
                }
            } else {
                return Err("Invalid number format".to_string());
            }
        }
        
        if self.current_char() == Some('.') {
            has_decimal = true;
            self.advance();
            
            if let Some(c) = self.current_char() {
                if !c.is_digit(10) {
                    return Err("Expected digits after decimal point".to_string());
                }
                
                while let Some(c) = self.current_char() {
                    if !c.is_digit(10) {
                        break;
                    }
                    self.advance();
                }
            }
        }
        
        if self.current_char() == Some('e') || self.current_char() == Some('E') {
            has_exponent = true;
            self.advance();
            
            if self.current_char() == Some('+') || self.current_char() == Some('-') {
                self.advance();
            }
            
            if let Some(c) = self.current_char() {
                if !c.is_digit(10) {
                    return Err("Expected digits in exponent".to_string());
                }
                
                while let Some(c) = self.current_char() {
                    if !c.is_digit(10) {
                        break;
                    }
                    self.advance();
                }
            }
        }
        
        let number_str: String = self.input[start..self.position].iter().collect();
        match number_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err("Invalid number format".to_string()),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.consume_char('[');
        self.skip_whitespace();
        
        let mut array = Vec::new();
        
        if self.current_char() == Some(']') {
            self.advance();
            return Ok(JsonValue::Array(array));
        }
        
        loop {
            let value = self.parse_value()?;
            array.push(value);
            
            self.skip_whitespace();
            
            if self.current_char() == Some(']') {
                self.advance();
                break;
            }
            
            if self.current_char() == Some(',') {
                self.advance();
                self.skip_whitespace();
                
                if self.current_char() == Some(']') {
                    return Err("Trailing comma in array".to_string());
                }
            } else {
                return Err("Expected ',' or ']' in array".to_string());
            }
        }
        
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.consume_char('{');
        self.skip_whitespace();
        
        let mut object = HashMap::new();
        
        if self.current_char() == Some('}') {
            self.advance();
            return Ok(JsonValue::Object(object));
        }
        
        loop {
            self.skip_whitespace();
            
            if self.current_char() != Some('"') {
                return Err("Expected string key in object".to_string());
            }
            
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => return Err("Expected string key".to_string()),
            };
            
            self.skip_whitespace();
            
            if self.current_char() != Some(':') {
                return Err("Expected ':' after object key".to_string());
            }
            self.advance();
            
            self.skip_whitespace();
            let value = self.parse_value()?;
            
            object.insert(key, value);
            
            self.skip_whitespace();
            
            if self.current_char() == Some('}') {
                self.advance();
                break;
            }
            
            if self.current_char() == Some(',') {
                self.advance();
                self.skip_whitespace();
                
                if self.current_char() == Some('}') {
                    return Err("Trailing comma in object".to_string());
                }
            } else {
                return Err("Expected ',' or '}' in object".to_string());
            }
        }
        
        Ok(JsonValue::Object(object))
    }

    fn current_char(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn advance(&mut self) {
        if self.position < self.input.len() {
            self.position += 1;
        }
    }

    fn consume_char(&mut self, expected: char) -> bool {
        if self.current_char() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume_str(&mut self, expected: &str) -> bool {
        let expected_chars: Vec<char> = expected.chars().collect();
        
        if self.position + expected_chars.len() > self.input.len() {
            return false;
        }
        
        for (i, &expected_char) in expected_chars.iter().enumerate() {
            if self.input[self.position + i] != expected_char {
                return false;
            }
        }
        
        self.position += expected_chars.len();
        true
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
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
        
        let mut parser = JsonParser::new("1.23e-4");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(0.000123)));
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
        let mut parser = JsonParser::new(r#"{"key": "value", "number": 42}"#);
        let mut expected = HashMap::new();
        expected.insert("key".to_string(), JsonValue::String("value".to_string()));
        expected.insert("number".to_string(), JsonValue::Number(42.0));
        assert_eq!(parser.parse(), Ok(JsonValue::Object(expected)));
    }
}