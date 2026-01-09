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

    fn parse_value(&mut self) -> Option<JsonValue> {
        self.skip_whitespace();
        if self.pos >= self.input.len() {
            return None;
        }

        let c = self.input.chars().nth(self.pos).unwrap();
        match c {
            'n' => self.parse_null(),
            't' | 'f' => self.parse_bool(),
            '"' => self.parse_string(),
            '[' => self.parse_array(),
            '{' => self.parse_object(),
            '0'..='9' | '-' => self.parse_number(),
            _ => None,
        }
    }

    fn parse_null(&mut self) -> Option<JsonValue> {
        if self.input[self.pos..].starts_with("null") {
            self.pos += 4;
            Some(JsonValue::Null)
        } else {
            None
        }
    }

    fn parse_bool(&mut self) -> Option<JsonValue> {
        if self.input[self.pos..].starts_with("true") {
            self.pos += 4;
            Some(JsonValue::Bool(true))
        } else if self.input[self.pos..].starts_with("false") {
            self.pos += 5;
            Some(JsonValue::Bool(false))
        } else {
            None
        }
    }

    fn parse_string(&mut self) -> Option<JsonValue> {
        self.pos += 1; // Skip opening quote
        let start = self.pos;
        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            if c == '"' {
                let s = self.input[start..self.pos].to_string();
                self.pos += 1; // Skip closing quote
                return Some(JsonValue::String(s));
            }
            self.pos += 1;
        }
        None
    }

    fn parse_number(&mut self) -> Option<JsonValue> {
        let start = self.pos;
        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            if !c.is_digit(10) && c != '.' && c != '-' && c != 'e' && c != 'E' {
                break;
            }
            self.pos += 1;
        }
        let num_str = &self.input[start..self.pos];
        match num_str.parse::<f64>() {
            Ok(num) => Some(JsonValue::Number(num)),
            Err(_) => None,
        }
    }

    fn parse_array(&mut self) -> Option<JsonValue> {
        self.pos += 1; // Skip '['
        let mut array = Vec::new();

        self.skip_whitespace();
        if self.pos < self.input.len() && self.input.chars().nth(self.pos).unwrap() == ']' {
            self.pos += 1;
            return Some(JsonValue::Array(array));
        }

        loop {
            if let Some(value) = self.parse_value() {
                array.push(value);
            } else {
                return None;
            }

            self.skip_whitespace();
            if self.pos >= self.input.len() {
                return None;
            }

            let c = self.input.chars().nth(self.pos).unwrap();
            if c == ']' {
                self.pos += 1;
                break;
            } else if c == ',' {
                self.pos += 1;
                self.skip_whitespace();
            } else {
                return None;
            }
        }

        Some(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Option<JsonValue> {
        self.pos += 1; // Skip '{'
        let mut object = HashMap::new();

        self.skip_whitespace();
        if self.pos < self.input.len() && self.input.chars().nth(self.pos).unwrap() == '}' {
            self.pos += 1;
            return Some(JsonValue::Object(object));
        }

        loop {
            self.skip_whitespace();
            if self.pos >= self.input.len() || self.input.chars().nth(self.pos).unwrap() != '"' {
                return None;
            }

            let key = match self.parse_string() {
                Some(JsonValue::String(s)) => s,
                _ => return None,
            };

            self.skip_whitespace();
            if self.pos >= self.input.len() || self.input.chars().nth(self.pos).unwrap() != ':' {
                return None;
            }
            self.pos += 1; // Skip ':'

            let value = match self.parse_value() {
                Some(v) => v,
                None => return None,
            };

            object.insert(key, value);

            self.skip_whitespace();
            if self.pos >= self.input.len() {
                return None;
            }

            let c = self.input.chars().nth(self.pos).unwrap();
            if c == '}' {
                self.pos += 1;
                break;
            } else if c == ',' {
                self.pos += 1;
                self.skip_whitespace();
            } else {
                return None;
            }
        }

        Some(JsonValue::Object(object))
    }

    pub fn parse(&mut self) -> Option<JsonValue> {
        self.parse_value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_null() {
        let mut parser = JsonParser::new("null");
        assert_eq!(parser.parse(), Some(JsonValue::Null));
    }

    #[test]
    fn test_parse_bool() {
        let mut parser = JsonParser::new("true");
        assert_eq!(parser.parse(), Some(JsonValue::Bool(true)));

        let mut parser = JsonParser::new("false");
        assert_eq!(parser.parse(), Some(JsonValue::Bool(false)));
    }

    #[test]
    fn test_parse_string() {
        let mut parser = JsonParser::new(r#""hello world""#);
        assert_eq!(
            parser.parse(),
            Some(JsonValue::String("hello world".to_string()))
        );
    }

    #[test]
    fn test_parse_number() {
        let mut parser = JsonParser::new("42.5");
        assert_eq!(parser.parse(), Some(JsonValue::Number(42.5)));
    }

    #[test]
    fn test_parse_array() {
        let mut parser = JsonParser::new("[1, 2, 3]");
        assert_eq!(
            parser.parse(),
            Some(JsonValue::Array(vec![
                JsonValue::Number(1.0),
                JsonValue::Number(2.0),
                JsonValue::Number(3.0),
            ]))
        );
    }

    #[test]
    fn test_parse_object() {
        let mut parser = JsonParser::new(r#"{"key": "value"}"#);
        let mut expected = HashMap::new();
        expected.insert("key".to_string(), JsonValue::String("value".to_string()));
        assert_eq!(parser.parse(), Some(JsonValue::Object(expected)));
    }
}