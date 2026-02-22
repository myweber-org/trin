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

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedCharacter(char, usize),
    UnexpectedEndOfInput,
    InvalidNumber,
    InvalidEscapeSequence,
    TrailingComma,
    ExpectedColon,
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

    fn consume(&mut self, expected: char) -> Result<(), ParseError> {
        self.skip_whitespace();
        if let Some(ch) = self.peek() {
            if ch == expected {
                self.position += 1;
                Ok(())
            } else {
                Err(ParseError::UnexpectedCharacter(ch, self.position))
            }
        } else {
            Err(ParseError::UnexpectedEndOfInput)
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, ParseError> {
        self.consume('"')?;
        let mut result = String::new();
        let mut escape = false;

        while self.position < self.input.len() {
            let ch = self.input[self.position];
            self.position += 1;

            if escape {
                match ch {
                    '"' => result.push('"'),
                    '\\' => result.push('\\'),
                    '/' => result.push('/'),
                    'b' => result.push('\u{0008}'),
                    'f' => result.push('\u{000C}'),
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    _ => return Err(ParseError::InvalidEscapeSequence),
                }
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                return Ok(JsonValue::String(result));
            } else {
                result.push(ch);
            }
        }

        Err(ParseError::UnexpectedEndOfInput)
    }

    fn parse_number(&mut self) -> Result<JsonValue, ParseError> {
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

        let number_str: String = self.input[start..self.position].iter().collect();
        match number_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(ParseError::InvalidNumber),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, ParseError> {
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
                    self.skip_whitespace();
                    if let Some(next_ch) = self.peek() {
                        if next_ch == ']' {
                            return Err(ParseError::TrailingComma);
                        }
                    }
                } else {
                    return Err(ParseError::UnexpectedCharacter(ch, self.position));
                }
            } else {
                return Err(ParseError::UnexpectedEndOfInput);
            }
        }

        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, ParseError> {
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
                _ => return Err(ParseError::UnexpectedCharacter(self.input[self.position], self.position)),
            };

            self.skip_whitespace();
            if let Some(ch) = self.peek() {
                if ch == ':' {
                    self.position += 1;
                } else {
                    return Err(ParseError::ExpectedColon);
                }
            } else {
                return Err(ParseError::UnexpectedEndOfInput);
            }

            let value = self.parse_value()?;
            map.insert(key, value);

            self.skip_whitespace();
            if let Some(ch) = self.peek() {
                if ch == '}' {
                    self.position += 1;
                    break;
                } else if ch == ',' {
                    self.position += 1;
                    self.skip_whitespace();
                    if let Some(next_ch) = self.peek() {
                        if next_ch == '}' {
                            return Err(ParseError::TrailingComma);
                        }
                    }
                } else {
                    return Err(ParseError::UnexpectedCharacter(ch, self.position));
                }
            } else {
                return Err(ParseError::UnexpectedEndOfInput);
            }
        }

        Ok(JsonValue::Object(map))
    }

    fn parse_value(&mut self) -> Result<JsonValue, ParseError> {
        self.skip_whitespace();

        if self.position >= self.input.len() {
            return Err(ParseError::UnexpectedEndOfInput);
        }

        let ch = self.input[self.position];
        match ch {
            'n' => {
                if self.position + 3 < self.input.len() 
                    && self.input[self.position..self.position + 4] == ['n', 'u', 'l', 'l'] {
                    self.position += 4;
                    Ok(JsonValue::Null)
                } else {
                    Err(ParseError::UnexpectedCharacter(ch, self.position))
                }
            }
            't' => {
                if self.position + 3 < self.input.len() 
                    && self.input[self.position..self.position + 4] == ['t', 'r', 'u', 'e'] {
                    self.position += 4;
                    Ok(JsonValue::Bool(true))
                } else {
                    Err(ParseError::UnexpectedCharacter(ch, self.position))
                }
            }
            'f' => {
                if self.position + 4 < self.input.len() 
                    && self.input[self.position..self.position + 5] == ['f', 'a', 'l', 's', 'e'] {
                    self.position += 5;
                    Ok(JsonValue::Bool(false))
                } else {
                    Err(ParseError::UnexpectedCharacter(ch, self.position))
                }
            }
            '"' => self.parse_string(),
            '[' => self.parse_array(),
            '{' => self.parse_object(),
            '-' | '0'..='9' => self.parse_number(),
            _ => Err(ParseError::UnexpectedCharacter(ch, self.position)),
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, ParseError> {
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.position < self.input.len() {
            Err(ParseError::UnexpectedCharacter(self.input[self.position], self.position))
        } else {
            Ok(result)
        }
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

    #[test]
    fn test_parse_object() {
        let mut parser = JsonParser::new(r#"{"key": "value"}"#);
        let mut map = HashMap::new();
        map.insert("key".to_string(), JsonValue::String("value".to_string()));
        assert_eq!(parser.parse(), Ok(JsonValue::Object(map)));
    }
}