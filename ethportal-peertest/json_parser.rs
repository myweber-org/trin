use std::collections::HashMap;
use std::str::Chars;

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Object(HashMap<String, JsonValue>),
    Array(Vec<JsonValue>),
}

pub struct JsonParser<'a> {
    chars: Chars<'a>,
    current: Option<char>,
}

impl<'a> JsonParser<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut chars = input.chars();
        let current = chars.next();
        JsonParser { chars, current }
    }

    fn advance(&mut self) {
        self.current = self.chars.next();
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        let mut result = String::new();
        self.advance(); // Skip opening quote
        while let Some(c) = self.current {
            match c {
                '"' => {
                    self.advance();
                    return Ok(result);
                }
                '\\' => {
                    self.advance();
                    if let Some(escaped) = self.current {
                        match escaped {
                            '"' => result.push('"'),
                            '\\' => result.push('\\'),
                            '/' => result.push('/'),
                            'b' => result.push('\u{0008}'),
                            'f' => result.push('\u{000C}'),
                            'n' => result.push('\n'),
                            'r' => result.push('\r'),
                            't' => result.push('\t'),
                            _ => return Err(format!("Invalid escape sequence: \\{}", escaped)),
                        }
                        self.advance();
                    }
                }
                _ => {
                    result.push(c);
                    self.advance();
                }
            }
        }
        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<f64, String> {
        let mut num_str = String::new();
        while let Some(c) = self.current {
            if c.is_digit(10) || c == '.' || c == '-' || c == '+' || c == 'e' || c == 'E' {
                num_str.push(c);
                self.advance();
            } else {
                break;
            }
        }
        num_str.parse::<f64>().map_err(|e| e.to_string())
    }

    fn parse_keyword(&mut self, keyword: &str, value: JsonValue) -> Result<JsonValue, String> {
        let mut chars = keyword.chars();
        while let Some(expected) = chars.next() {
            match self.current {
                Some(c) if c == expected => self.advance(),
                _ => return Err(format!("Expected keyword '{}'", keyword)),
            }
        }
        Ok(value)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        match self.current {
            Some('"') => self.parse_string().map(JsonValue::String),
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('t') => self.parse_keyword("true", JsonValue::Boolean(true)),
            Some('f') => self.parse_keyword("false", JsonValue::Boolean(false)),
            Some('n') => self.parse_keyword("null", JsonValue::Null),
            Some(c) if c.is_digit(10) || c == '-' => self.parse_number().map(JsonValue::Number),
            _ => Err("Invalid JSON value".to_string()),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        let mut map = HashMap::new();
        self.advance(); // Skip '{'
        self.skip_whitespace();

        if self.current == Some('}') {
            self.advance();
            return Ok(JsonValue::Object(map));
        }

        loop {
            self.skip_whitespace();
            if self.current != Some('"') {
                return Err("Expected string key".to_string());
            }
            let key = self.parse_string()?;
            self.skip_whitespace();
            if self.current != Some(':') {
                return Err("Expected ':' after key".to_string());
            }
            self.advance();
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();
            match self.current {
                Some(',') => {
                    self.advance();
                    continue;
                }
                Some('}') => {
                    self.advance();
                    break;
                }
                _ => return Err("Expected ',' or '}' in object".to_string()),
            }
        }
        Ok(JsonValue::Object(map))
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        let mut arr = Vec::new();
        self.advance(); // Skip '['
        self.skip_whitespace();

        if self.current == Some(']') {
            self.advance();
            return Ok(JsonValue::Array(arr));
        }

        loop {
            let value = self.parse_value()?;
            arr.push(value);
            self.skip_whitespace();
            match self.current {
                Some(',') => {
                    self.advance();
                    continue;
                }
                Some(']') => {
                    self.advance();
                    break;
                }
                _ => return Err("Expected ',' or ']' in array".to_string()),
            }
        }
        Ok(JsonValue::Array(arr))
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.current.is_some() {
            return Err("Trailing characters after JSON".to_string());
        }
        Ok(result)
    }
}

pub fn parse_json(input: &str) -> Result<JsonValue, String> {
    let mut parser = JsonParser::new(input);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_object() {
        let json = r#"{"name": "John", "age": 30, "active": true}"#;
        let result = parse_json(json);
        assert!(result.is_ok());
        if let Ok(JsonValue::Object(map)) = result {
            assert_eq!(map.get("name"), Some(&JsonValue::String("John".to_string())));
            assert_eq!(map.get("age"), Some(&JsonValue::Number(30.0)));
            assert_eq!(map.get("active"), Some(&JsonValue::Boolean(true)));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_parse_array() {
        let json = r#"[1, 2, 3, "hello", false]"#;
        let result = parse_json(json);
        assert!(result.is_ok());
        if let Ok(JsonValue::Array(arr)) = result {
            assert_eq!(arr.len(), 5);
            assert_eq!(arr[0], JsonValue::Number(1.0));
            assert_eq!(arr[3], JsonValue::String("hello".to_string()));
            assert_eq!(arr[4], JsonValue::Boolean(false));
        } else {
            panic!("Expected array");
        }
    }
}