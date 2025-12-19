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

#[derive(Debug)]
pub struct ParseError {
    message: String,
    position: usize,
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

    pub fn parse(&mut self) -> Result<JsonValue, ParseError> {
        self.skip_whitespace();
        let value = self.parse_value()?;
        self.skip_whitespace();
        
        if self.position < self.input.len() {
            return Err(self.error("Unexpected trailing characters"));
        }
        
        Ok(value)
    }

    fn parse_value(&mut self) -> Result<JsonValue, ParseError> {
        self.skip_whitespace();
        
        match self.peek_char() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(c) if c.is_digit(10) || c == '-' => self.parse_number(),
            _ => Err(self.error("Invalid JSON value")),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, ParseError> {
        if self.consume_str("null") {
            Ok(JsonValue::Null)
        } else {
            Err(self.error("Expected 'null'"))
        }
    }

    fn parse_bool(&mut self) -> Result<JsonValue, ParseError> {
        if self.consume_str("true") {
            Ok(JsonValue::Bool(true))
        } else if self.consume_str("false") {
            Ok(JsonValue::Bool(false))
        } else {
            Err(self.error("Expected boolean value"))
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, ParseError> {
        self.consume_char('"');
        let mut result = String::new();
        
        while let Some(c) = self.next_char() {
            match c {
                '"' => break,
                '\\' => {
                    if let Some(escaped) = self.next_char() {
                        match escaped {
                            '"' => result.push('"'),
                            '\\' => result.push('\\'),
                            '/' => result.push('/'),
                            'b' => result.push('\u{0008}'),
                            'f' => result.push('\u{000C}'),
                            'n' => result.push('\n'),
                            'r' => result.push('\r'),
                            't' => result.push('\t'),
                            _ => return Err(self.error("Invalid escape sequence")),
                        }
                    }
                }
                _ => result.push(c),
            }
        }
        
        Ok(JsonValue::String(result))
    }

    fn parse_number(&mut self) -> Result<JsonValue, ParseError> {
        let start = self.position;
        let mut has_dot = false;
        
        if self.peek_char() == Some('-') {
            self.next_char();
        }
        
        while let Some(c) = self.peek_char() {
            if c.is_digit(10) {
                self.next_char();
            } else if c == '.' && !has_dot {
                has_dot = true;
                self.next_char();
            } else {
                break;
            }
        }
        
        let number_str: String = self.input[start..self.position].iter().collect();
        match number_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(self.error("Invalid number format")),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, ParseError> {
        self.consume_char('[');
        self.skip_whitespace();
        
        let mut array = Vec::new();
        
        if self.peek_char() == Some(']') {
            self.next_char();
            return Ok(JsonValue::Array(array));
        }
        
        loop {
            let value = self.parse_value()?;
            array.push(value);
            
            self.skip_whitespace();
            match self.peek_char() {
                Some(',') => {
                    self.next_char();
                    self.skip_whitespace();
                }
                Some(']') => {
                    self.next_char();
                    break;
                }
                _ => return Err(self.error("Expected ',' or ']' in array")),
            }
        }
        
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, ParseError> {
        self.consume_char('{');
        self.skip_whitespace();
        
        let mut object = HashMap::new();
        
        if self.peek_char() == Some('}') {
            self.next_char();
            return Ok(JsonValue::Object(object));
        }
        
        loop {
            self.skip_whitespace();
            
            if self.peek_char() != Some('"') {
                return Err(self.error("Expected string key in object"));
            }
            
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => unreachable!(),
            };
            
            self.skip_whitespace();
            
            if self.peek_char() != Some(':') {
                return Err(self.error("Expected ':' after object key"));
            }
            self.next_char();
            
            let value = self.parse_value()?;
            object.insert(key, value);
            
            self.skip_whitespace();
            match self.peek_char() {
                Some(',') => {
                    self.next_char();
                    self.skip_whitespace();
                }
                Some('}') => {
                    self.next_char();
                    break;
                }
                _ => return Err(self.error("Expected ',' or '}' in object")),
            }
        }
        
        Ok(JsonValue::Object(object))
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn next_char(&mut self) -> Option<char> {
        if self.position < self.input.len() {
            let c = self.input[self.position];
            self.position += 1;
            Some(c)
        } else {
            None
        }
    }

    fn consume_char(&mut self, expected: char) -> bool {
        if self.peek_char() == Some(expected) {
            self.next_char();
            true
        } else {
            false
        }
    }

    fn consume_str(&mut self, expected: &str) -> bool {
        let start = self.position;
        for ch in expected.chars() {
            if !self.consume_char(ch) {
                self.position = start;
                return false;
            }
        }
        true
    }

    fn error(&self, message: &str) -> ParseError {
        ParseError {
            message: message.to_string(),
            position: self.position,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_null() {
        let mut parser = JsonParser::new("null");
        assert_eq!(parser.parse().unwrap(), JsonValue::Null);
    }

    #[test]
    fn test_parse_bool() {
        let mut parser = JsonParser::new("true");
        assert_eq!(parser.parse().unwrap(), JsonValue::Bool(true));
        
        let mut parser = JsonParser::new("false");
        assert_eq!(parser.parse().unwrap(), JsonValue::Bool(false));
    }

    #[test]
    fn test_parse_number() {
        let mut parser = JsonParser::new("42");
        assert_eq!(parser.parse().unwrap(), JsonValue::Number(42.0));
        
        let mut parser = JsonParser::new("-3.14");
        assert_eq!(parser.parse().unwrap(), JsonValue::Number(-3.14));
    }

    #[test]
    fn test_parse_string() {
        let mut parser = JsonParser::new(r#""hello world""#);
        assert_eq!(parser.parse().unwrap(), JsonValue::String("hello world".to_string()));
    }

    #[test]
    fn test_parse_array() {
        let mut parser = JsonParser::new("[1, 2, 3]");
        let result = parser.parse().unwrap();
        
        if let JsonValue::Array(arr) = result {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], JsonValue::Number(1.0));
            assert_eq!(arr[1], JsonValue::Number(2.0));
            assert_eq!(arr[2], JsonValue::Number(3.0));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_parse_object() {
        let mut parser = JsonParser::new(r#"{"key": "value", "number": 42}"#);
        let result = parser.parse().unwrap();
        
        if let JsonValue::Object(obj) = result {
            assert_eq!(obj.len(), 2);
            assert_eq!(obj.get("key"), Some(&JsonValue::String("value".to_string())));
            assert_eq!(obj.get("number"), Some(&JsonValue::Number(42.0)));
        } else {
            panic!("Expected object");
        }
    }
}