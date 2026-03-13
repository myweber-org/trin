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
    pos: usize,
}

impl JsonParser {
    pub fn new(input: &str) -> Self {
        JsonParser {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn consume(&mut self, expected: char) -> Result<(), String> {
        match self.peek() {
            Some(ch) if ch == expected => {
                self.pos += 1;
                Ok(())
            }
            Some(ch) => Err(format!("Expected '{}', found '{}'", expected, ch)),
            None => Err(format!("Expected '{}', found EOF", expected)),
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.consume('"')?;
        let mut result = String::new();

        while let Some(ch) = self.peek() {
            match ch {
                '"' => {
                    self.pos += 1;
                    return Ok(result);
                }
                '\\' => {
                    self.pos += 1;
                    let escaped = self.peek().ok_or("Unexpected EOF after escape")?;
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
                    self.pos += 1;
                }
                _ => {
                    result.push(ch);
                    self.pos += 1;
                }
            }
        }

        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<f64, String> {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '.' || ch == '-' || ch == '+' || ch == 'e' || ch == 'E' {
                self.pos += 1;
            } else {
                break;
            }
        }

        let slice: String = self.input[start..self.pos].iter().collect();
        slice.parse().map_err(|e| format!("Invalid number: {}", e))
    }

    fn parse_array(&mut self) -> Result<Vec<JsonValue>, String> {
        self.consume('[')?;
        self.skip_whitespace();

        let mut array = Vec::new();

        if let Some(']') = self.peek() {
            self.pos += 1;
            return Ok(array);
        }

        loop {
            self.skip_whitespace();
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();

            match self.peek() {
                Some(',') => {
                    self.pos += 1;
                    continue;
                }
                Some(']') => {
                    self.pos += 1;
                    break;
                }
                Some(ch) => return Err(format!("Expected ',' or ']', found '{}'", ch)),
                None => return Err("Unexpected EOF in array".to_string()),
            }
        }

        Ok(array)
    }

    fn parse_object(&mut self) -> Result<HashMap<String, JsonValue>, String> {
        self.consume('{')?;
        self.skip_whitespace();

        let mut object = HashMap::new();

        if let Some('}') = self.peek() {
            self.pos += 1;
            return Ok(object);
        }

        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            self.skip_whitespace();
            self.consume(':')?;
            self.skip_whitespace();
            let value = self.parse_value()?;
            object.insert(key, value);
            self.skip_whitespace();

            match self.peek() {
                Some(',') => {
                    self.pos += 1;
                    continue;
                }
                Some('}') => {
                    self.pos += 1;
                    break;
                }
                Some(ch) => return Err(format!("Expected ',' or '}}', found '{}'", ch)),
                None => return Err("Unexpected EOF in object".to_string()),
            }
        }

        Ok(object)
    }

    pub fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();

        match self.peek() {
            Some('n') => {
                self.pos += 4;
                if &self.input[self.pos-4..self.pos] == ['n', 'u', 'l', 'l'] {
                    Ok(JsonValue::Null)
                } else {
                    Err("Invalid null value".to_string())
                }
            }
            Some('t') => {
                self.pos += 4;
                if &self.input[self.pos-4..self.pos] == ['t', 'r', 'u', 'e'] {
                    Ok(JsonValue::Bool(true))
                } else {
                    Err("Invalid boolean value".to_string())
                }
            }
            Some('f') => {
                self.pos += 5;
                if &self.input[self.pos-5..self.pos] == ['f', 'a', 'l', 's', 'e'] {
                    Ok(JsonValue::Bool(false))
                } else {
                    Err("Invalid boolean value".to_string())
                }
            }
            Some('"') => self.parse_string().map(JsonValue::String),
            Some('[') => self.parse_array().map(JsonValue::Array),
            Some('{') => self.parse_object().map(JsonValue::Object),
            Some(ch) if ch.is_ascii_digit() || ch == '-' => self.parse_number().map(JsonValue::Number),
            Some(ch) => Err(format!("Unexpected character: {}", ch)),
            None => Err("Unexpected EOF".to_string()),
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            Err("Trailing characters after JSON value".to_string())
        } else {
            Ok(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_json() {
        let json = r#"{"name": "test", "value": 42.5, "active": true}"#;
        let mut parser = JsonParser::new(json);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_array() {
        let json = r#"[1, 2, 3, "hello", false]"#;
        let mut parser = JsonParser::new(json);
        let result = parser.parse();
        assert!(result.is_ok());
    }
}