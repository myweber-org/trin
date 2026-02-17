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
        if let Some(ch) = self.peek() {
            if ch == expected {
                self.position += 1;
                Ok(())
            } else {
                Err(format!("Expected '{}', found '{}'", expected, ch))
            }
        } else {
            Err("Unexpected end of input".to_string())
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.consume('"')?;
        let mut result = String::new();
        
        while self.position < self.input.len() {
            let ch = self.input[self.position];
            self.position += 1;
            
            if ch == '"' {
                return Ok(JsonValue::String(result));
            } else if ch == '\\' {
                if self.position >= self.input.len() {
                    return Err("Unterminated escape sequence".to_string());
                }
                let next_char = self.input[self.position];
                self.position += 1;
                match next_char {
                    '"' => result.push('"'),
                    '\\' => result.push('\\'),
                    '/' => result.push('/'),
                    'b' => result.push('\x08'),
                    'f' => result.push('\x0c'),
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    _ => return Err(format!("Invalid escape sequence: \\{}", next_char)),
                }
            } else {
                result.push(ch);
            }
        }
        
        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
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
        
        let num_str: String = self.input[start..self.position].iter().collect();
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(format!("Invalid number: {}", num_str)),
        }
    }

    fn parse_keyword(&mut self, keyword: &str, value: JsonValue) -> Result<JsonValue, String> {
        let start = self.position;
        let end = start + keyword.len();
        
        if end <= self.input.len() {
            let slice: String = self.input[start..end].iter().collect();
            if slice == keyword {
                self.position = end;
                return Ok(value);
            }
        }
        
        Err(format!("Expected keyword '{}'", keyword))
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.consume('[')?;
        self.skip_whitespace();
        
        let mut array = Vec::new();
        
        if let Some(ch) = self.peek() {
            if ch == ']' {
                self.position += 1;
                return Ok(JsonValue::Array(array));
            }
        }
        
        loop {
            let value = self.parse_value()?;
            array.push(value);
            
            self.skip_whitespace();
            if let Some(ch) = self.peek() {
                if ch == ']' {
                    self.position += 1;
                    break;
                } else if ch == ',' {
                    self.position += 1;
                    continue;
                } else {
                    return Err(format!("Expected ',' or ']', found '{}'", ch));
                }
            } else {
                return Err("Unexpected end of input".to_string());
            }
        }
        
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.consume('{')?;
        self.skip_whitespace();
        
        let mut map = HashMap::new();
        
        if let Some(ch) = self.peek() {
            if ch == '}' {
                self.position += 1;
                return Ok(JsonValue::Object(map));
            }
        }
        
        loop {
            let key = match self.parse_value()? {
                JsonValue::String(s) => s,
                _ => return Err("Object keys must be strings".to_string()),
            };
            
            self.skip_whitespace();
            self.consume(':')?;
            
            let value = self.parse_value()?;
            map.insert(key, value);
            
            self.skip_whitespace();
            if let Some(ch) = self.peek() {
                if ch == '}' {
                    self.position += 1;
                    break;
                } else if ch == ',' {
                    self.position += 1;
                    continue;
                } else {
                    return Err(format!("Expected ',' or '}}', found '{}'", ch));
                }
            } else {
                return Err("Unexpected end of input".to_string());
            }
        }
        
        Ok(JsonValue::Object(map))
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        
        if let Some(ch) = self.peek() {
            match ch {
                '"' => self.parse_string(),
                '[' => self.parse_array(),
                '{' => self.parse_object(),
                't' => self.parse_keyword("true", JsonValue::Bool(true)),
                'f' => self.parse_keyword("false", JsonValue::Bool(false)),
                'n' => self.parse_keyword("null", JsonValue::Null),
                '-' | '0'..='9' => self.parse_number(),
                _ => Err(format!("Unexpected character: '{}'", ch)),
            }
        } else {
            Err("Unexpected end of input".to_string())
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        let result = self.parse_value()?;
        self.skip_whitespace();
        
        if self.position < self.input.len() {
            return Err("Trailing characters after JSON value".to_string());
        }
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string() {
        let mut parser = JsonParser::new(r#""hello world""#);
        assert_eq!(parser.parse(), Ok(JsonValue::String("hello world".to_string())));
    }

    #[test]
    fn test_parse_number() {
        let mut parser = JsonParser::new("42.5");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(42.5)));
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
}