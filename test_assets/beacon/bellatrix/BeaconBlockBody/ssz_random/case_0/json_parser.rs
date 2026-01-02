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
        let result = self.parse_value()?;
        self.skip_whitespace();
        
        if self.position < self.input.len() {
            return Err(ParseError {
                message: "Unexpected trailing characters".to_string(),
                position: self.position,
            });
        }
        
        Ok(result)
    }

    fn parse_value(&mut self) -> Result<JsonValue, ParseError> {
        match self.peek_char() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(c) if c.is_digit(10) || c == '-' => self.parse_number(),
            _ => Err(ParseError {
                message: "Unexpected character".to_string(),
                position: self.position,
            }),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, ParseError> {
        if self.consume_str("null") {
            Ok(JsonValue::Null)
        } else {
            Err(ParseError {
                message: "Expected 'null'".to_string(),
                position: self.position,
            })
        }
    }

    fn parse_bool(&mut self) -> Result<JsonValue, ParseError> {
        if self.consume_str("true") {
            Ok(JsonValue::Bool(true))
        } else if self.consume_str("false") {
            Ok(JsonValue::Bool(false))
        } else {
            Err(ParseError {
                message: "Expected boolean value".to_string(),
                position: self.position,
            })
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, ParseError> {
        let start = self.position;
        let mut has_decimal = false;
        
        if self.consume_char('-') {
            // Negative number
        }
        
        while let Some(c) = self.peek_char() {
            if c.is_digit(10) {
                self.consume_char(c);
            } else if c == '.' && !has_decimal {
                has_decimal = true;
                self.consume_char(c);
            } else {
                break;
            }
        }
        
        let number_str: String = self.input[start..self.position].iter().collect();
        match number_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(ParseError {
                message: "Invalid number format".to_string(),
                position: start,
            }),
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, ParseError> {
        self.consume_char('"');
        let mut result = String::new();
        
        while let Some(c) = self.peek_char() {
            if c == '"' {
                self.consume_char('"');
                return Ok(JsonValue::String(result));
            } else if c == '\\' {
                self.consume_char('\\');
                if let Some(escaped) = self.peek_char() {
                    match escaped {
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        '/' => result.push('/'),
                        'b' => result.push('\u{0008}'),
                        'f' => result.push('\u{000C}'),
                        'n' => result.push('\n'),
                        'r' => result.push('\r'),
                        't' => result.push('\t'),
                        _ => return Err(ParseError {
                            message: "Invalid escape sequence".to_string(),
                            position: self.position - 1,
                        }),
                    }
                    self.consume_char(escaped);
                }
            } else {
                result.push(c);
                self.consume_char(c);
            }
        }
        
        Err(ParseError {
            message: "Unterminated string".to_string(),
            position: self.position,
        })
    }

    fn parse_array(&mut self) -> Result<JsonValue, ParseError> {
        self.consume_char('[');
        self.skip_whitespace();
        
        let mut array = Vec::new();
        
        if self.peek_char() == Some(']') {
            self.consume_char(']');
            return Ok(JsonValue::Array(array));
        }
        
        loop {
            let value = self.parse_value()?;
            array.push(value);
            
            self.skip_whitespace();
            match self.peek_char() {
                Some(',') => {
                    self.consume_char(',');
                    self.skip_whitespace();
                }
                Some(']') => {
                    self.consume_char(']');
                    break;
                }
                _ => {
                    return Err(ParseError {
                        message: "Expected ',' or ']' in array".to_string(),
                        position: self.position,
                    });
                }
            }
        }
        
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, ParseError> {
        self.consume_char('{');
        self.skip_whitespace();
        
        let mut object = HashMap::new();
        
        if self.peek_char() == Some('}') {
            self.consume_char('}');
            return Ok(JsonValue::Object(object));
        }
        
        loop {
            self.skip_whitespace();
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => unreachable!(),
            };
            
            self.skip_whitespace();
            if !self.consume_char(':') {
                return Err(ParseError {
                    message: "Expected ':' after object key".to_string(),
                    position: self.position,
                });
            }
            
            self.skip_whitespace();
            let value = self.parse_value()?;
            object.insert(key, value);
            
            self.skip_whitespace();
            match self.peek_char() {
                Some(',') => {
                    self.consume_char(',');
                    self.skip_whitespace();
                }
                Some('}') => {
                    self.consume_char('}');
                    break;
                }
                _ => {
                    return Err(ParseError {
                        message: "Expected ',' or '}' in object".to_string(),
                        position: self.position,
                    });
                }
            }
        }
        
        Ok(JsonValue::Object(object))
    }

    fn peek_char(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn consume_char(&mut self, expected: char) -> bool {
        if self.peek_char() == Some(expected) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn consume_str(&mut self, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        if self.position + chars.len() <= self.input.len() {
            for (i, &c) in chars.iter().enumerate() {
                if self.input[self.position + i] != c {
                    return false;
                }
            }
            self.position += chars.len();
            true
        } else {
            false
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.position += 1;
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
        
        let mut parser = JsonParser::new(r#""escape\nsequence""#);
        assert_eq!(parser.parse().unwrap(), JsonValue::String("escape\nsequence".to_string()));
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
        let mut parser = JsonParser::new(r#"{"key": "value", "number": 42}"#);
        let mut expected = HashMap::new();
        expected.insert("key".to_string(), JsonValue::String("value".to_string()));
        expected.insert("number".to_string(), JsonValue::Number(42.0));
        assert_eq!(parser.parse().unwrap(), JsonValue::Object(expected));
    }
}