
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

#[derive(Debug)]
struct ParseError {
    message: String,
    position: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JSON parse error at position {}: {}", self.position, self.message)
    }
}

impl Error for ParseError {}

struct JsonParser {
    input: Vec<char>,
    position: usize,
}

impl JsonParser {
    fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
        }
    }

    fn parse(&mut self) -> Result<JsonValue, ParseError> {
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
        self.skip_whitespace();
        
        match self.peek_char() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(c) if c.is_digit(10) || c == '-' => self.parse_number(),
            _ => Err(ParseError {
                message: "Invalid JSON value".to_string(),
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
        
        if !self.consume_digits() {
            return Err(ParseError {
                message: "Invalid number format".to_string(),
                position: start,
            });
        }
        
        if self.consume_char('.') {
            has_decimal = true;
            if !self.consume_digits() {
                return Err(ParseError {
                    message: "Invalid decimal number".to_string(),
                    position: self.position,
                });
            }
        }
        
        if self.consume_char('e') || self.consume_char('E') {
            self.consume_char('-');
            self.consume_char('+');
            if !self.consume_digits() {
                return Err(ParseError {
                    message: "Invalid exponent format".to_string(),
                    position: self.position,
                });
            }
        }
        
        let number_str: String = self.input[start..self.position].iter().collect();
        match number_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(ParseError {
                message: "Invalid number value".to_string(),
                position: start,
            }),
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, ParseError> {
        if !self.consume_char('"') {
            return Err(ParseError {
                message: "Expected opening quote".to_string(),
                position: self.position,
            });
        }
        
        let mut result = String::new();
        while let Some(c) = self.next_char() {
            match c {
                '"' => return Ok(JsonValue::String(result)),
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
                            'u' => {
                                let hex_str: String = (0..4)
                                    .filter_map(|_| self.next_char())
                                    .collect();
                                
                                if hex_str.len() != 4 {
                                    return Err(ParseError {
                                        message: "Invalid Unicode escape sequence".to_string(),
                                        position: self.position - 4,
                                    });
                                }
                                
                                match u32::from_str_radix(&hex_str, 16) {
                                    Ok(code_point) => {
                                        if let Some(ch) = std::char::from_u32(code_point) {
                                            result.push(ch);
                                        } else {
                                            return Err(ParseError {
                                                message: "Invalid Unicode code point".to_string(),
                                                position: self.position - 4,
                                            });
                                        }
                                    }
                                    Err(_) => {
                                        return Err(ParseError {
                                            message: "Invalid hex digits in Unicode escape".to_string(),
                                            position: self.position - 4,
                                        });
                                    }
                                }
                            }
                            _ => return Err(ParseError {
                                message: "Invalid escape sequence".to_string(),
                                position: self.position - 1,
                            }),
                        }
                    } else {
                        return Err(ParseError {
                            message: "Unterminated escape sequence".to_string(),
                            position: self.position,
                        });
                    }
                }
                _ => result.push(c),
            }
        }
        
        Err(ParseError {
            message: "Unterminated string".to_string(),
            position: self.position,
        })
    }

    fn parse_array(&mut self) -> Result<JsonValue, ParseError> {
        if !self.consume_char('[') {
            return Err(ParseError {
                message: "Expected '['".to_string(),
                position: self.position,
            });
        }
        
        self.skip_whitespace();
        let mut array = Vec::new();
        
        if self.consume_char(']') {
            return Ok(JsonValue::Array(array));
        }
        
        loop {
            let value = self.parse_value()?;
            array.push(value);
            
            self.skip_whitespace();
            if self.consume_char(']') {
                break;
            }
            
            if !self.consume_char(',') {
                return Err(ParseError {
                    message: "Expected ',' or ']'".to_string(),
                    position: self.position,
                });
            }
            
            self.skip_whitespace();
        }
        
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, ParseError> {
        if !self.consume_char('{') {
            return Err(ParseError {
                message: "Expected '{'".to_string(),
                position: self.position,
            });
        }
        
        self.skip_whitespace();
        let mut object = HashMap::new();
        
        if self.consume_char('}') {
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
                    message: "Expected ':'".to_string(),
                    position: self.position,
                });
            }
            
            let value = self.parse_value()?;
            object.insert(key, value);
            
            self.skip_whitespace();
            if self.consume_char('}') {
                break;
            }
            
            if !self.consume_char(',') {
                return Err(ParseError {
                    message: "Expected ',' or '}'".to_string(),
                    position: self.position,
                });
            }
        }
        
        Ok(JsonValue::Object(object))
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

    fn consume_char(&mut self, expected: char) -> bool {
        if self.peek_char() == Some(expected) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn consume_str(&mut self, expected: &str) -> bool {
        let expected_chars: Vec<char> = expected.chars().collect();
        if self.position + expected_chars.len() <= self.input.len() {
            for (i, &expected_char) in expected_chars.iter().enumerate() {
                if self.input[self.position + i] != expected_char {
                    return false;
                }
            }
            self.position += expected_chars.len();
            true
        } else {
            false
        }
    }

    fn consume_digits(&mut self) -> bool {
        let start = self.position;
        while let Some(c) = self.peek_char() {
            if c.is_digit(10) {
                self.position += 1;
            } else {
                break;
            }
        }
        self.position > start
    }
}

pub fn validate_json(input: &str) -> Result<JsonValue, ParseError> {
    let mut parser = JsonParser::new(input);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_null() {
        assert_eq!(validate_json("null").unwrap(), JsonValue::Null);
    }

    #[test]
    fn test_valid_boolean() {
        assert_eq!(validate_json("true").unwrap(), JsonValue::Bool(true));
        assert_eq!(validate_json("false").unwrap(), JsonValue::Bool(false));
    }

    #[test]
    fn test_valid_number() {
        assert_eq!(validate_json("42").unwrap(), JsonValue::Number(42.0));
        assert_eq!(validate_json("-3.14").unwrap(), JsonValue::Number(-3.14));
        assert_eq!(validate_json("1.5e2").unwrap(), JsonValue::Number(150.0));
    }

    #[test]
    fn test_valid_string() {
        assert_eq!(
            validate_json(r#""hello""#).unwrap(),
            JsonValue::String("hello".to_string())
        );
        assert_eq!(
            validate_json(r#""escape\"test""#).unwrap(),
            JsonValue::String("escape\"test".to_string())
        );
    }

    #[test]
    fn test_valid_array() {
        let result = validate_json("[1, 2, 3]").unwrap();
        if let JsonValue::Array(arr) = result {
            assert_eq!(arr.len(), 3);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_valid_object() {
        let result = validate_json(r#"{"key": "value"}"#).unwrap();
        if let JsonValue::Object(obj) = result {
            assert_eq!(obj.len(), 1);
            assert_eq!(obj.get("key"), Some(&JsonValue::String("value".to_string())));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_invalid_json() {
        assert!(validate_json("invalid").is_err());
        assert!(validate_json("{").is_err());
        assert!(validate_json("[1, 2, 3").is_err());
    }
}