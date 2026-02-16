use std::collections::HashMap;
use std::error::Error;
use std::fmt;

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

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JSON parse error at position {}: {}", self.position, self.message)
    }
}

impl Error for ParseError {}

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
        match self.peek_char() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(c) if c.is_digit(10) || c == '-' => self.parse_number(),
            _ => Err(self.error("Expected JSON value")),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, ParseError> {
        self.expect("null")?;
        Ok(JsonValue::Null)
    }

    fn parse_bool(&mut self) -> Result<JsonValue, ParseError> {
        if self.starts_with("true") {
            self.advance(4);
            Ok(JsonValue::Bool(true))
        } else if self.starts_with("false") {
            self.advance(5);
            Ok(JsonValue::Bool(false))
        } else {
            Err(self.error("Expected boolean value"))
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, ParseError> {
        let start = self.position;
        let mut has_decimal = false;
        
        if self.peek_char() == Some('-') {
            self.advance(1);
        }
        
        while let Some(c) = self.peek_char() {
            if c.is_digit(10) {
                self.advance(1);
            } else if c == '.' && !has_decimal {
                has_decimal = true;
                self.advance(1);
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

    fn parse_string(&mut self) -> Result<JsonValue, ParseError> {
        self.expect("\"")?;
        let mut result = String::new();
        
        while let Some(c) = self.next_char() {
            match c {
                '"' => return Ok(JsonValue::String(result)),
                '\\' => {
                    let escaped = self.parse_escape_sequence()?;
                    result.push(escaped);
                }
                _ => result.push(c),
            }
        }
        
        Err(self.error("Unterminated string"))
    }

    fn parse_escape_sequence(&mut self) -> Result<char, ParseError> {
        match self.next_char() {
            Some('"') => Ok('"'),
            Some('\\') => Ok('\\'),
            Some('/') => Ok('/'),
            Some('b') => Ok('\x08'),
            Some('f') => Ok('\x0c'),
            Some('n') => Ok('\n'),
            Some('r') => Ok('\r'),
            Some('t') => Ok('\t'),
            _ => Err(self.error("Invalid escape sequence")),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, ParseError> {
        self.expect("[")?;
        self.skip_whitespace();
        
        let mut array = Vec::new();
        
        if self.peek_char() == Some(']') {
            self.advance(1);
            return Ok(JsonValue::Array(array));
        }
        
        loop {
            let value = self.parse_value()?;
            array.push(value);
            
            self.skip_whitespace();
            
            match self.peek_char() {
                Some(',') => {
                    self.advance(1);
                    self.skip_whitespace();
                }
                Some(']') => {
                    self.advance(1);
                    break;
                }
                _ => return Err(self.error("Expected ',' or ']' in array")),
            }
        }
        
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, ParseError> {
        self.expect("{")?;
        self.skip_whitespace();
        
        let mut object = HashMap::new();
        
        if self.peek_char() == Some('}') {
            self.advance(1);
            return Ok(JsonValue::Object(object));
        }
        
        loop {
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => unreachable!(),
            };
            
            self.skip_whitespace();
            self.expect(":")?;
            self.skip_whitespace();
            
            let value = self.parse_value()?;
            object.insert(key, value);
            
            self.skip_whitespace();
            
            match self.peek_char() {
                Some(',') => {
                    self.advance(1);
                    self.skip_whitespace();
                }
                Some('}') => {
                    self.advance(1);
                    break;
                }
                _ => return Err(self.error("Expected ',' or '}' in object")),
            }
        }
        
        Ok(JsonValue::Object(object))
    }

    fn expect(&mut self, expected: &str) -> Result<(), ParseError> {
        for expected_char in expected.chars() {
            match self.next_char() {
                Some(c) if c == expected_char => continue,
                _ => return Err(self.error(&format!("Expected '{}'", expected))),
            }
        }
        Ok(())
    }

    fn starts_with(&self, prefix: &str) -> bool {
        self.input[self.position..]
            .iter()
            .zip(prefix.chars())
            .all(|(&a, b)| a == b)
    }

    fn peek_char(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn next_char(&mut self) -> Option<char> {
        let c = self.peek_char();
        if c.is_some() {
            self.position += 1;
        }
        c
    }

    fn advance(&mut self, n: usize) {
        self.position += n;
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.advance(1);
            } else {
                break;
            }
        }
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
        let mut parser = JsonParser::new("\"hello\"");
        assert_eq!(parser.parse().unwrap(), JsonValue::String("hello".to_string()));
    }

    #[test]
    fn test_parse_array() {
        let mut parser = JsonParser::new("[1, 2, 3]");
        let expected = JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::Number(2.0),
            JsonValue::Number(3.0),
        ]);
        assert_eq!(parser.parse().unwrap(), expected);
    }

    #[test]
    fn test_parse_object() {
        let mut parser = JsonParser::new(r#"{"key": "value"}"#);
        let mut expected_map = HashMap::new();
        expected_map.insert("key".to_string(), JsonValue::String("value".to_string()));
        let expected = JsonValue::Object(expected_map);
        assert_eq!(parser.parse().unwrap(), expected);
    }
}