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
pub enum ParseError {
    UnexpectedEnd,
    InvalidToken,
    ExpectedColon,
    ExpectedComma,
    TrailingCharacters,
}

pub struct JsonParser {
    input: String,
    pos: usize,
}

impl JsonParser {
    pub fn new(input: String) -> Self {
        JsonParser { input, pos: 0 }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            if c.is_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn parse_string(&mut self) -> Result<String, ParseError> {
        if self.input.chars().nth(self.pos) != Some('"') {
            return Err(ParseError::InvalidToken);
        }
        self.pos += 1;
        let start = self.pos;
        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            if c == '"' {
                let result = self.input[start..self.pos].to_string();
                self.pos += 1;
                return Ok(result);
            }
            self.pos += 1;
        }
        Err(ParseError::UnexpectedEnd)
    }

    fn parse_number(&mut self) -> Result<f64, ParseError> {
        let start = self.pos;
        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            if c.is_digit(10) || c == '.' || c == '-' || c == 'e' || c == 'E' {
                self.pos += 1;
            } else {
                break;
            }
        }
        let num_str = &self.input[start..self.pos];
        num_str.parse::<f64>()
            .map_err(|_| ParseError::InvalidToken)
    }

    fn parse_array(&mut self) -> Result<Vec<JsonValue>, ParseError> {
        if self.input.chars().nth(self.pos) != Some('[') {
            return Err(ParseError::InvalidToken);
        }
        self.pos += 1;
        self.skip_whitespace();
        
        let mut array = Vec::new();
        if self.input.chars().nth(self.pos) == Some(']') {
            self.pos += 1;
            return Ok(array);
        }

        loop {
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();
            
            match self.input.chars().nth(self.pos) {
                Some(',') => {
                    self.pos += 1;
                    self.skip_whitespace();
                }
                Some(']') => {
                    self.pos += 1;
                    break;
                }
                _ => return Err(ParseError::ExpectedComma),
            }
        }
        Ok(array)
    }

    fn parse_object(&mut self) -> Result<HashMap<String, JsonValue>, ParseError> {
        if self.input.chars().nth(self.pos) != Some('{') {
            return Err(ParseError::InvalidToken);
        }
        self.pos += 1;
        self.skip_whitespace();
        
        let mut map = HashMap::new();
        if self.input.chars().nth(self.pos) == Some('}') {
            self.pos += 1;
            return Ok(map);
        }

        loop {
            let key = self.parse_string()?;
            self.skip_whitespace();
            
            if self.input.chars().nth(self.pos) != Some(':') {
                return Err(ParseError::ExpectedColon);
            }
            self.pos += 1;
            self.skip_whitespace();
            
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();
            
            match self.input.chars().nth(self.pos) {
                Some(',') => {
                    self.pos += 1;
                    self.skip_whitespace();
                }
                Some('}') => {
                    self.pos += 1;
                    break;
                }
                _ => return Err(ParseError::ExpectedComma),
            }
        }
        Ok(map)
    }

    fn parse_value(&mut self) -> Result<JsonValue, ParseError> {
        self.skip_whitespace();
        if self.pos >= self.input.len() {
            return Err(ParseError::UnexpectedEnd);
        }

        let c = self.input.chars().nth(self.pos).unwrap();
        match c {
            '"' => {
                let s = self.parse_string()?;
                Ok(JsonValue::String(s))
            }
            '[' => {
                let arr = self.parse_array()?;
                Ok(JsonValue::Array(arr))
            }
            '{' => {
                let obj = self.parse_object()?;
                Ok(JsonValue::Object(obj))
            }
            't' if self.input[self.pos..].starts_with("true") => {
                self.pos += 4;
                Ok(JsonValue::Bool(true))
            }
            'f' if self.input[self.pos..].starts_with("false") => {
                self.pos += 5;
                Ok(JsonValue::Bool(false))
            }
            'n' if self.input[self.pos..].starts_with("null") => {
                self.pos += 4;
                Ok(JsonValue::Null)
            }
            '-' | '0'..='9' => {
                let num = self.parse_number()?;
                Ok(JsonValue::Number(num))
            }
            _ => Err(ParseError::InvalidToken),
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, ParseError> {
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            return Err(ParseError::TrailingCharacters);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string() {
        let mut parser = JsonParser::new(r#""hello world""#.to_string());
        assert_eq!(parser.parse(), Ok(JsonValue::String("hello world".to_string())));
    }

    #[test]
    fn test_parse_number() {
        let mut parser = JsonParser::new("42.5".to_string());
        assert_eq!(parser.parse(), Ok(JsonValue::Number(42.5)));
    }

    #[test]
    fn test_parse_array() {
        let mut parser = JsonParser::new("[1, 2, 3]".to_string());
        let expected = JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::Number(2.0),
            JsonValue::Number(3.0),
        ]);
        assert_eq!(parser.parse(), Ok(expected));
    }
}