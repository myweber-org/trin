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
    input: String,
    pos: usize,
}

impl JsonParser {
    pub fn new(input: &str) -> Self {
        JsonParser {
            input: input.to_string(),
            pos: 0,
        }
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

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        if self.pos >= self.input.len() {
            return Err("Unexpected end of input".to_string());
        }

        let c = self.input.chars().nth(self.pos).unwrap();
        match c {
            'n' => self.parse_null(),
            't' | 'f' => self.parse_bool(),
            '"' => self.parse_string(),
            '[' => self.parse_array(),
            '{' => self.parse_object(),
            '-' | '0'..='9' => self.parse_number(),
            _ => Err(format!("Unexpected character: {}", c)),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        if self.input[self.pos..].starts_with("null") {
            self.pos += 4;
            Ok(JsonValue::Null)
        } else {
            Err("Expected 'null'".to_string())
        }
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.input[self.pos..].starts_with("true") {
            self.pos += 4;
            Ok(JsonValue::Bool(true))
        } else if self.input[self.pos..].starts_with("false") {
            self.pos += 5;
            Ok(JsonValue::Bool(false))
        } else {
            Err("Expected 'true' or 'false'".to_string())
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.pos += 1; // Skip opening quote
        let start = self.pos;
        let mut result = String::new();

        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            if c == '"' {
                self.pos += 1;
                return Ok(JsonValue::String(result));
            } else if c == '\\' {
                self.pos += 1;
                if self.pos >= self.input.len() {
                    return Err("Unterminated string".to_string());
                }
                let escaped = self.input.chars().nth(self.pos).unwrap();
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
            } else {
                result.push(c);
                self.pos += 1;
            }
        }

        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        let mut has_dot = false;
        let mut has_exponent = false;

        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            match c {
                '0'..='9' => self.pos += 1,
                '.' => {
                    if has_dot || has_exponent {
                        return Err("Invalid number format".to_string());
                    }
                    has_dot = true;
                    self.pos += 1;
                }
                'e' | 'E' => {
                    if has_exponent {
                        return Err("Invalid number format".to_string());
                    }
                    has_exponent = true;
                    self.pos += 1;
                    if self.pos < self.input.len() {
                        let next = self.input.chars().nth(self.pos).unwrap();
                        if next == '+' || next == '-' {
                            self.pos += 1;
                        }
                    }
                }
                '+' | '-' => {
                    if self.pos != start {
                        return Err("Invalid number format".to_string());
                    }
                    self.pos += 1;
                }
                _ => break,
            }
        }

        let num_str = &self.input[start..self.pos];
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err("Invalid number".to_string()),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.pos += 1; // Skip '['
        let mut array = Vec::new();

        self.skip_whitespace();
        if self.pos < self.input.len() && self.input.chars().nth(self.pos).unwrap() == ']' {
            self.pos += 1;
            return Ok(JsonValue::Array(array));
        }

        loop {
            let value = self.parse_value()?;
            array.push(value);

            self.skip_whitespace();
            if self.pos >= self.input.len() {
                return Err("Unterminated array".to_string());
            }

            let c = self.input.chars().nth(self.pos).unwrap();
            if c == ']' {
                self.pos += 1;
                break;
            } else if c == ',' {
                self.pos += 1;
                self.skip_whitespace();
            } else {
                return Err(format!("Expected ',' or ']', found: {}", c));
            }
        }

        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.pos += 1; // Skip '{'
        let mut object = HashMap::new();

        self.skip_whitespace();
        if self.pos < self.input.len() && self.input.chars().nth(self.pos).unwrap() == '}' {
            self.pos += 1;
            return Ok(JsonValue::Object(object));
        }

        loop {
            self.skip_whitespace();
            if self.input.chars().nth(self.pos).unwrap() != '"' {
                return Err("Expected string key".to_string());
            }

            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => unreachable!(),
            };

            self.skip_whitespace();
            if self.pos >= self.input.len() || self.input.chars().nth(self.pos).unwrap() != ':' {
                return Err("Expected ':' after key".to_string());
            }
            self.pos += 1;

            let value = self.parse_value()?;
            object.insert(key, value);

            self.skip_whitespace();
            if self.pos >= self.input.len() {
                return Err("Unterminated object".to_string());
            }

            let c = self.input.chars().nth(self.pos).unwrap();
            if c == '}' {
                self.pos += 1;
                break;
            } else if c == ',' {
                self.pos += 1;
                self.skip_whitespace();
            } else {
                return Err(format!("Expected ',' or '}}', found: {}", c));
            }
        }

        Ok(JsonValue::Object(object))
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            return Err("Trailing characters after JSON value".to_string());
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_null() {
        let mut parser = JsonParser::new("null");
        assert_eq!(parser.parse(), Ok(JsonValue::Null));
    }

    #[test]
    fn test_parse_bool() {
        let mut parser = JsonParser::new("true");
        assert_eq!(parser.parse(), Ok(JsonValue::Bool(true)));

        let mut parser = JsonParser::new("false");
        assert_eq!(parser.parse(), Ok(JsonValue::Bool(false)));
    }

    #[test]
    fn test_parse_number() {
        let mut parser = JsonParser::new("42");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(42.0)));

        let mut parser = JsonParser::new("-3.14");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(-3.14)));

        let mut parser = JsonParser::new("1.23e-4");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(1.23e-4)));
    }

    #[test]
    fn test_parse_string() {
        let mut parser = JsonParser::new(r#""hello world""#);
        assert_eq!(
            parser.parse(),
            Ok(JsonValue::String("hello world".to_string()))
        );

        let mut parser = JsonParser::new(r#""escape\"test""#);
        assert_eq!(
            parser.parse(),
            Ok(JsonValue::String("escape\"test".to_string()))
        );
    }

    #[test]
    fn test_parse_array() {
        let mut parser = JsonParser::new("[1, 2, 3]");
        assert_eq!(
            parser.parse(),
            Ok(JsonValue::Array(vec![
                JsonValue::Number(1.0),
                JsonValue::Number(2.0),
                JsonValue::Number(3.0),
            ]))
        );

        let mut parser = JsonParser::new("[]");
        assert_eq!(parser.parse(), Ok(JsonValue::Array(vec![])));
    }

    #[test]
    fn test_parse_object() {
        let mut parser = JsonParser::new(r#"{"key": "value"}"#);
        let mut expected = HashMap::new();
        expected.insert("key".to_string(), JsonValue::String("value".to_string()));
        assert_eq!(parser.parse(), Ok(JsonValue::Object(expected)));

        let mut parser = JsonParser::new("{}");
        assert_eq!(parser.parse(), Ok(JsonValue::Object(HashMap::new())));
    }
}use std::collections::HashMap;
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

pub struct JsonParser<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> JsonParser<'a> {
    pub fn new(input: &'a str) -> Self {
        JsonParser {
            chars: input.chars().peekable(),
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        match self.chars.peek() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(c) if c.is_digit(10) || *c == '-' => self.parse_number(),
            _ => Err("Unexpected character".to_string()),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        let expected = "null";
        for ch in expected.chars() {
            match self.chars.next() {
                Some(c) if c == ch => continue,
                _ => return Err("Invalid null value".to_string()),
            }
        }
        Ok(JsonValue::Null)
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        let mut buffer = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_alphabetic() {
                buffer.push(self.chars.next().unwrap());
            } else {
                break;
            }
        }

        match buffer.as_str() {
            "true" => Ok(JsonValue::Bool(true)),
            "false" => Ok(JsonValue::Bool(false)),
            _ => Err("Invalid boolean value".to_string()),
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        let mut result = String::new();
        self.chars.next(); // Skip opening quote

        while let Some(&c) = self.chars.peek() {
            match c {
                '"' => {
                    self.chars.next(); // Skip closing quote
                    return Ok(JsonValue::String(result));
                }
                '\\' => {
                    self.chars.next(); // Skip backslash
                    if let Some(escaped) = self.chars.next() {
                        match escaped {
                            'n' => result.push('\n'),
                            't' => result.push('\t'),
                            'r' => result.push('\r'),
                            '\\' => result.push('\\'),
                            '"' => result.push('"'),
                            _ => return Err("Invalid escape sequence".to_string()),
                        }
                    }
                }
                _ => result.push(self.chars.next().unwrap()),
            }
        }

        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let mut buffer = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_digit(10) || c == '.' || c == '-' || c == '+' || c == 'e' || c == 'E' {
                buffer.push(self.chars.next().unwrap());
            } else {
                break;
            }
        }

        match buffer.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err("Invalid number format".to_string()),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        let mut array = Vec::new();
        self.chars.next(); // Skip '['

        self.skip_whitespace();
        if let Some(']') = self.chars.peek() {
            self.chars.next(); // Skip ']'
            return Ok(JsonValue::Array(array));
        }

        loop {
            let value = self.parse_value()?;
            array.push(value);

            self.skip_whitespace();
            match self.chars.peek() {
                Some(',') => {
                    self.chars.next(); // Skip ','
                    self.skip_whitespace();
                }
                Some(']') => {
                    self.chars.next(); // Skip ']'
                    break;
                }
                _ => return Err("Expected ',' or ']' in array".to_string()),
            }
        }

        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        let mut map = HashMap::new();
        self.chars.next(); // Skip '{'

        self.skip_whitespace();
        if let Some('}') = self.chars.peek() {
            self.chars.next(); // Skip '}'
            return Ok(JsonValue::Object(map));
        }

        loop {
            self.skip_whitespace();
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => return Err("Object key must be a string".to_string()),
            };

            self.skip_whitespace();
            match self.chars.peek() {
                Some(':') => {
                    self.chars.next(); // Skip ':'
                }
                _ => return Err("Expected ':' after object key".to_string()),
            }

            let value = self.parse_value()?;
            map.insert(key, value);

            self.skip_whitespace();
            match self.chars.peek() {
                Some(',') => {
                    self.chars.next(); // Skip ','
                    self.skip_whitespace();
                }
                Some('}') => {
                    self.chars.next(); // Skip '}'
                    break;
                }
                _ => return Err("Expected ',' or '}' in object".to_string()),
            }
        }

        Ok(JsonValue::Object(map))
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.chars.peek().is_some() {
            return Err("Trailing characters after JSON value".to_string());
        }
        Ok(result)
    }
}

pub fn parse_json(input: &str) -> Result<JsonValue, String> {
    let mut parser = JsonParser::new(input);
    parser.parse()
}