use std::collections::HashMap;

#[derive(Debug, PartialEq)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

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

    fn parse(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.position < self.input.len() {
            return Err("Unexpected trailing characters".to_string());
        }
        Ok(result)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        match self.peek_char() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(c) if c.is_digit(10) || c == '-' => self.parse_number(),
            _ => Err("Invalid JSON value".to_string()),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        if self.consume_str("null") {
            Ok(JsonValue::Null)
        } else {
            Err("Expected 'null'".to_string())
        }
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.consume_str("true") {
            Ok(JsonValue::Bool(true))
        } else if self.consume_str("false") {
            Ok(JsonValue::Bool(false))
        } else {
            Err("Expected boolean value".to_string())
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.position;
        if self.consume_char('-') {
            // Optional minus sign
        }
        while let Some(c) = self.peek_char() {
            if c.is_digit(10) {
                self.consume_char(c);
            } else {
                break;
            }
        }
        if self.consume_char('.') {
            while let Some(c) = self.peek_char() {
                if c.is_digit(10) {
                    self.consume_char(c);
                } else {
                    break;
                }
            }
        }
        let number_str: String = self.input[start..self.position].iter().collect();
        match number_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err("Invalid number".to_string()),
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        if !self.consume_char('"') {
            return Err("Expected opening quote".to_string());
        }
        let mut result = String::new();
        while let Some(c) = self.next_char() {
            if c == '"' {
                return Ok(JsonValue::String(result));
            } else if c == '\\' {
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
                        _ => return Err("Invalid escape sequence".to_string()),
                    }
                } else {
                    return Err("Unterminated escape sequence".to_string());
                }
            } else {
                result.push(c);
            }
        }
        Err("Unterminated string".to_string())
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        if !self.consume_char('[') {
            return Err("Expected '['".to_string());
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
                return Err("Expected ',' or ']'".to_string());
            }
            self.skip_whitespace();
        }
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        if !self.consume_char('{') {
            return Err("Expected '{'".to_string());
        }
        self.skip_whitespace();
        let mut map = HashMap::new();
        if self.consume_char('}') {
            return Ok(JsonValue::Object(map));
        }
        loop {
            self.skip_whitespace();
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => return Err("Object key must be a string".to_string()),
            };
            self.skip_whitespace();
            if !self.consume_char(':') {
                return Err("Expected ':'".to_string());
            }
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();
            if self.consume_char('}') {
                break;
            }
            if !self.consume_char(',') {
                return Err("Expected ',' or '}'".to_string());
            }
        }
        Ok(JsonValue::Object(map))
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.consume_char(c);
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn next_char(&mut self) -> Option<char> {
        if self.position < self.input.len() {
            let c = self.input[self.position];
            self.position += 1;
            Some(c)
        } else {
            None
        }
    }

    fn consume_char(&mut self, expected: char) -> bool {
        if let Some(c) = self.peek_char() {
            if c == expected {
                self.position += 1;
                return true;
            }
        }
        false
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
}

fn parse_json(input: &str) -> Result<JsonValue, String> {
    let mut parser = JsonParser::new(input);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_null() {
        assert_eq!(parse_json("null"), Ok(JsonValue::Null));
    }

    #[test]
    fn test_parse_bool() {
        assert_eq!(parse_json("true"), Ok(JsonValue::Bool(true)));
        assert_eq!(parse_json("false"), Ok(JsonValue::Bool(false)));
    }

    #[test]
    fn test_parse_number() {
        assert_eq!(parse_json("42"), Ok(JsonValue::Number(42.0)));
        assert_eq!(parse_json("-3.14"), Ok(JsonValue::Number(-3.14)));
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(
            parse_json("\"hello\""),
            Ok(JsonValue::String("hello".to_string()))
        );
    }

    #[test]
    fn test_parse_array() {
        assert_eq!(
            parse_json("[1, 2, 3]"),
            Ok(JsonValue::Array(vec![
                JsonValue::Number(1.0),
                JsonValue::Number(2.0),
                JsonValue::Number(3.0),
            ]))
        );
    }

    #[test]
    fn test_parse_object() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), JsonValue::String("value".to_string()));
        assert_eq!(parse_json("{\"key\": \"value\"}"), Ok(JsonValue::Object(map)));
    }
}