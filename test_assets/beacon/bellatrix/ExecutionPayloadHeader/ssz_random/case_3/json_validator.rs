use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

#[derive(Debug, PartialEq)]
pub enum JsonError {
    UnexpectedCharacter(char, usize),
    UnexpectedEnd,
    InvalidNumber,
    InvalidEscapeSequence,
    TrailingCharacters,
}

pub struct JsonParser<'a> {
    chars: Peekable<Chars<'a>>,
    position: usize,
}

impl<'a> JsonParser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
            position: 0,
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, JsonError> {
        let value = self.parse_value()?;
        self.skip_whitespace();
        if self.chars.peek().is_some() {
            return Err(JsonError::TrailingCharacters);
        }
        Ok(value)
    }

    fn parse_value(&mut self) -> Result<JsonValue, JsonError> {
        self.skip_whitespace();
        match self.chars.peek() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(c) if c.is_digit(10) || *c == '-' => self.parse_number(),
            _ => Err(JsonError::UnexpectedEnd),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, JsonError> {
        self.expect("null")?;
        Ok(JsonValue::Null)
    }

    fn parse_bool(&mut self) -> Result<JsonValue, JsonError> {
        if self.starts_with("true") {
            self.expect("true")?;
            Ok(JsonValue::Bool(true))
        } else if self.starts_with("false") {
            self.expect("false")?;
            Ok(JsonValue::Bool(false))
        } else {
            Err(JsonError::UnexpectedCharacter(
                *self.chars.peek().unwrap(),
                self.position,
            ))
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, JsonError> {
        self.consume('"')?;
        let mut result = String::new();
        
        while let Some(c) = self.chars.next() {
            self.position += 1;
            match c {
                '"' => return Ok(JsonValue::String(result)),
                '\\' => {
                    let escaped = self.parse_escape_sequence()?;
                    result.push(escaped);
                }
                c if c.is_control() => {
                    return Err(JsonError::UnexpectedCharacter(c, self.position));
                }
                c => result.push(c),
            }
        }
        
        Err(JsonError::UnexpectedEnd)
    }

    fn parse_escape_sequence(&mut self) -> Result<char, JsonError> {
        match self.chars.next() {
            Some('"') => {
                self.position += 1;
                Ok('"')
            }
            Some('\\') => {
                self.position += 1;
                Ok('\\')
            }
            Some('/') => {
                self.position += 1;
                Ok('/')
            }
            Some('b') => {
                self.position += 1;
                Ok('\x08')
            }
            Some('f') => {
                self.position += 1;
                Ok('\x0C')
            }
            Some('n') => {
                self.position += 1;
                Ok('\n')
            }
            Some('r') => {
                self.position += 1;
                Ok('\r')
            }
            Some('t') => {
                self.position += 1;
                Ok('\t')
            }
            Some('u') => {
                self.position += 1;
                self.parse_unicode_escape()
            }
            _ => Err(JsonError::InvalidEscapeSequence),
        }
    }

    fn parse_unicode_escape(&mut self) -> Result<char, JsonError> {
        let hex_digits: String = self.chars.by_ref().take(4).collect();
        self.position += 4;
        
        if hex_digits.len() != 4 {
            return Err(JsonError::InvalidEscapeSequence);
        }
        
        u32::from_str_radix(&hex_digits, 16)
            .ok()
            .and_then(char::from_u32)
            .ok_or(JsonError::InvalidEscapeSequence)
    }

    fn parse_number(&mut self) -> Result<JsonValue, JsonError> {
        let mut num_str = String::new();
        
        if let Some('-') = self.chars.peek() {
            num_str.push(self.chars.next().unwrap());
            self.position += 1;
        }
        
        if let Some('0') = self.chars.peek() {
            num_str.push(self.chars.next().unwrap());
            self.position += 1;
        } else if let Some(c) = self.chars.peek() {
            if c.is_digit(10) && *c != '0' {
                while let Some(c) = self.chars.peek() {
                    if c.is_digit(10) {
                        num_str.push(self.chars.next().unwrap());
                        self.position += 1;
                    } else {
                        break;
                    }
                }
            }
        }
        
        if let Some('.') = self.chars.peek() {
            num_str.push(self.chars.next().unwrap());
            self.position += 1;
            
            while let Some(c) = self.chars.peek() {
                if c.is_digit(10) {
                    num_str.push(self.chars.next().unwrap());
                    self.position += 1;
                } else {
                    break;
                }
            }
        }
        
        if let Some('e') | Some('E') = self.chars.peek() {
            num_str.push(self.chars.next().unwrap());
            self.position += 1;
            
            if let Some('+') | Some('-') = self.chars.peek() {
                num_str.push(self.chars.next().unwrap());
                self.position += 1;
            }
            
            while let Some(c) = self.chars.peek() {
                if c.is_digit(10) {
                    num_str.push(self.chars.next().unwrap());
                    self.position += 1;
                } else {
                    break;
                }
            }
        }
        
        num_str
            .parse::<f64>()
            .map(JsonValue::Number)
            .map_err(|_| JsonError::InvalidNumber)
    }

    fn parse_array(&mut self) -> Result<JsonValue, JsonError> {
        self.consume('[')?;
        self.skip_whitespace();
        
        let mut array = Vec::new();
        
        if let Some(']') = self.chars.peek() {
            self.chars.next();
            self.position += 1;
            return Ok(JsonValue::Array(array));
        }
        
        loop {
            let value = self.parse_value()?;
            array.push(value);
            
            self.skip_whitespace();
            match self.chars.peek() {
                Some(',') => {
                    self.chars.next();
                    self.position += 1;
                    self.skip_whitespace();
                }
                Some(']') => {
                    self.chars.next();
                    self.position += 1;
                    break;
                }
                Some(c) => return Err(JsonError::UnexpectedCharacter(*c, self.position)),
                None => return Err(JsonError::UnexpectedEnd),
            }
        }
        
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, JsonError> {
        self.consume('{')?;
        self.skip_whitespace();
        
        let mut object = HashMap::new();
        
        if let Some('}') = self.chars.peek() {
            self.chars.next();
            self.position += 1;
            return Ok(JsonValue::Object(object));
        }
        
        loop {
            self.skip_whitespace();
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => unreachable!(),
            };
            
            self.skip_whitespace();
            self.consume(':')?;
            
            let value = self.parse_value()?;
            object.insert(key, value);
            
            self.skip_whitespace();
            match self.chars.peek() {
                Some(',') => {
                    self.chars.next();
                    self.position += 1;
                    self.skip_whitespace();
                }
                Some('}') => {
                    self.chars.next();
                    self.position += 1;
                    break;
                }
                Some(c) => return Err(JsonError::UnexpectedCharacter(*c, self.position)),
                None => return Err(JsonError::UnexpectedEnd),
            }
        }
        
        Ok(JsonValue::Object(object))
    }

    fn expect(&mut self, expected: &str) -> Result<(), JsonError> {
        for ch in expected.chars() {
            self.consume(ch)?;
        }
        Ok(())
    }

    fn starts_with(&mut self, prefix: &str) -> bool {
        let mut peek_chars = self.chars.clone();
        for expected_ch in prefix.chars() {
            match peek_chars.next() {
                Some(ch) if ch == expected_ch => continue,
                _ => return false,
            }
        }
        true
    }

    fn consume(&mut self, expected: char) -> Result<(), JsonError> {
        match self.chars.next() {
            Some(ch) if ch == expected => {
                self.position += 1;
                Ok(())
            }
            Some(ch) => Err(JsonError::UnexpectedCharacter(ch, self.position)),
            None => Err(JsonError::UnexpectedEnd),
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.chars.peek() {
            if c.is_whitespace() {
                self.chars.next();
                self.position += 1;
            } else {
                break;
            }
        }
    }
}

pub fn validate_json(input: &str) -> Result<JsonValue, JsonError> {
    let mut parser = JsonParser::new(input);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let json = r#"{"name": "John", "age": 30, "active": true}"#;
        assert!(validate_json(json).is_ok());
    }

    #[test]
    fn test_invalid_json() {
        let json = r#"{"name": "John", "age": 30,}"#;
        assert!(validate_json(json).is_err());
    }

    #[test]
    fn test_nested_structures() {
        let json = r#"{"users": [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]}"#;
        assert!(validate_json(json).is_ok());
    }
}