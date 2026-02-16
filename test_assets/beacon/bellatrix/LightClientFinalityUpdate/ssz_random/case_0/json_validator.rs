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
        JsonParser {
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
        
        if self.position >= self.input.len() {
            return Err(ParseError {
                message: "Unexpected end of input".to_string(),
                position: self.position,
            });
        }

        match self.input[self.position] {
            'n' => self.parse_null(),
            't' | 'f' => self.parse_bool(),
            '"' => self.parse_string(),
            '[' => self.parse_array(),
            '{' => self.parse_object(),
            '-' | '0'..='9' => self.parse_number(),
            _ => Err(ParseError {
                message: format!("Unexpected character '{}'", self.input[self.position]),
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
                message: "Expected 'true' or 'false'".to_string(),
                position: self.position,
            })
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, ParseError> {
        self.consume_char('"');
        let start = self.position;
        
        while self.position < self.input.len() && self.input[self.position] != '"' {
            if self.input[self.position] == '\\' {
                self.position += 1;
            }
            self.position += 1;
        }

        if self.position >= self.input.len() {
            return Err(ParseError {
                message: "Unterminated string".to_string(),
                position: start,
            });
        }

        let end = self.position;
        self.consume_char('"');
        
        let raw_str: String = self.input[start..end].iter().collect();
        let decoded = self.unescape_string(&raw_str)?;
        
        Ok(JsonValue::String(decoded))
    }

    fn parse_number(&mut self) -> Result<JsonValue, ParseError> {
        let start = self.position;
        
        if self.input[self.position] == '-' {
            self.position += 1;
        }

        while self.position < self.input.len() && self.input[self.position].is_ascii_digit() {
            self.position += 1;
        }

        if self.position < self.input.len() && self.input[self.position] == '.' {
            self.position += 1;
            while self.position < self.input.len() && self.input[self.position].is_ascii_digit() {
                self.position += 1;
            }
        }

        let num_str: String = self.input[start..self.position].iter().collect();
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(ParseError {
                message: format!("Invalid number '{}'", num_str),
                position: start,
            }),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, ParseError> {
        self.consume_char('[');
        self.skip_whitespace();
        
        let mut array = Vec::new();
        
        if self.input[self.position] != ']' {
            loop {
                let value = self.parse_value()?;
                array.push(value);
                
                self.skip_whitespace();
                if self.position >= self.input.len() {
                    return Err(ParseError {
                        message: "Unterminated array".to_string(),
                        position: self.position,
                    });
                }
                
                if self.input[self.position] == ']' {
                    break;
                }
                
                if self.input[self.position] != ',' {
                    return Err(ParseError {
                        message: "Expected ',' or ']'".to_string(),
                        position: self.position,
                    });
                }
                
                self.position += 1;
                self.skip_whitespace();
            }
        }
        
        self.consume_char(']');
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, ParseError> {
        self.consume_char('{');
        self.skip_whitespace();
        
        let mut object = HashMap::new();
        
        if self.input[self.position] != '}' {
            loop {
                let key = match self.parse_value()? {
                    JsonValue::String(s) => s,
                    _ => return Err(ParseError {
                        message: "Object key must be a string".to_string(),
                        position: self.position,
                    }),
                };
                
                self.skip_whitespace();
                if self.position >= self.input.len() || self.input[self.position] != ':' {
                    return Err(ParseError {
                        message: "Expected ':' after object key".to_string(),
                        position: self.position,
                    });
                }
                
                self.position += 1;
                self.skip_whitespace();
                
                let value = self.parse_value()?;
                object.insert(key, value);
                
                self.skip_whitespace();
                if self.position >= self.input.len() {
                    return Err(ParseError {
                        message: "Unterminated object".to_string(),
                        position: self.position,
                    });
                }
                
                if self.input[self.position] == '}' {
                    break;
                }
                
                if self.input[self.position] != ',' {
                    return Err(ParseError {
                        message: "Expected ',' or '}'".to_string(),
                        position: self.position,
                    });
                }
                
                self.position += 1;
                self.skip_whitespace();
            }
        }
        
        self.consume_char('}');
        Ok(JsonValue::Object(object))
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && self.input[self.position].is_whitespace() {
            self.position += 1;
        }
    }

    fn consume_char(&mut self, expected: char) {
        if self.position < self.input.len() && self.input[self.position] == expected {
            self.position += 1;
        }
    }

    fn consume_str(&mut self, expected: &str) -> bool {
        let chars: Vec<char> = expected.chars().collect();
        
        if self.position + chars.len() <= self.input.len() {
            for (i, &ch) in chars.iter().enumerate() {
                if self.input[self.position + i] != ch {
                    return false;
                }
            }
            self.position += chars.len();
            true
        } else {
            false
        }
    }

    fn unescape_string(&self, input: &str) -> Result<String, ParseError> {
        let mut result = String::new();
        let mut chars = input.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '\\' {
                match chars.next() {
                    Some('"') => result.push('"'),
                    Some('\\') => result.push('\\'),
                    Some('/') => result.push('/'),
                    Some('b') => result.push('\x08'),
                    Some('f') => result.push('\x0c'),
                    Some('n') => result.push('\n'),
                    Some('r') => result.push('\r'),
                    Some('t') => result.push('\t'),
                    Some('u') => {
                        let hex_str: String = chars.by_ref().take(4).collect();
                        if hex_str.len() != 4 {
                            return Err(ParseError {
                                message: "Invalid Unicode escape sequence".to_string(),
                                position: self.position,
                            });
                        }
                        match u32::from_str_radix(&hex_str, 16) {
                            Ok(code) => match char::from_u32(code) {
                                Some(unicode_char) => result.push(unicode_char),
                                None => return Err(ParseError {
                                    message: "Invalid Unicode code point".to_string(),
                                    position: self.position,
                                }),
                            },
                            Err(_) => return Err(ParseError {
                                message: "Invalid hex digits in Unicode escape".to_string(),
                                position: self.position,
                            }),
                        }
                    }
                    Some(c) => return Err(ParseError {
                        message: format!("Invalid escape sequence '\\{}'", c),
                        position: self.position,
                    }),
                    None => return Err(ParseError {
                        message: "Incomplete escape sequence".to_string(),
                        position: self.position,
                    }),
                }
            } else {
                result.push(ch);
            }
        }
        
        Ok(result)
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
    fn test_valid_json() {
        let json = r#"{"name": "test", "value": 42, "active": true}"#;
        let result = validate_json(json);
        assert!(result.is_ok());
        
        if let Ok(JsonValue::Object(obj)) = result {
            assert_eq!(obj.get("name"), Some(&JsonValue::String("test".to_string())));
            assert_eq!(obj.get("value"), Some(&JsonValue::Number(42.0)));
            assert_eq!(obj.get("active"), Some(&JsonValue::Bool(true)));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_invalid_json() {
        let json = r#"{"name": test}"#;
        let result = validate_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_object() {
        let json = r#"{}"#;
        let result = validate_json(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_array() {
        let json = r#"[1, 2, "three", true]"#;
        let result = validate_json(json);
        assert!(result.is_ok());
    }
}