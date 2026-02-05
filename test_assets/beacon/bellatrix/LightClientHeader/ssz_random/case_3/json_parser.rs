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

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            return Err("Unexpected trailing characters".to_string());
        }
        Ok(result)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        match self.peek() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(ch) if ch.is_digit(10) || ch == '-' => self.parse_number(),
            _ => Err("Invalid JSON value".to_string()),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        let expected = "null";
        for ch in expected.chars() {
            self.consume(ch)?;
        }
        Ok(JsonValue::Null)
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.input[self.pos..].starts_with(&['t', 'r', 'u', 'e']) {
            for ch in "true".chars() {
                self.consume(ch)?;
            }
            Ok(JsonValue::Bool(true))
        } else if self.input[self.pos..].starts_with(&['f', 'a', 'l', 's', 'e']) {
            for ch in "false".chars() {
                self.consume(ch)?;
            }
            Ok(JsonValue::Bool(false))
        } else {
            Err("Invalid boolean value".to_string())
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        if self.peek() == Some('-') {
            self.pos += 1;
        }

        while let Some(ch) = self.peek() {
            if ch.is_digit(10) {
                self.pos += 1;
            } else {
                break;
            }
        }

        if self.peek() == Some('.') {
            self.pos += 1;
            while let Some(ch) = self.peek() {
                if ch.is_digit(10) {
                    self.pos += 1;
                } else {
                    break;
                }
            }
        }

        let number_str: String = self.input[start..self.pos].iter().collect();
        match number_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err("Invalid number format".to_string()),
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.consume('"')?;
        let mut result = String::new();

        while let Some(ch) = self.peek() {
            if ch == '"' {
                break;
            } else if ch == '\\' {
                self.pos += 1;
                let escaped = self.parse_escape()?;
                result.push(escaped);
            } else {
                result.push(ch);
                self.pos += 1;
            }
        }

        self.consume('"')?;
        Ok(JsonValue::String(result))
    }

    fn parse_escape(&mut self) -> Result<char, String> {
        match self.peek() {
            Some('"') => {
                self.pos += 1;
                Ok('"')
            }
            Some('\\') => {
                self.pos += 1;
                Ok('\\')
            }
            Some('/') => {
                self.pos += 1;
                Ok('/')
            }
            Some('b') => {
                self.pos += 1;
                Ok('\x08')
            }
            Some('f') => {
                self.pos += 1;
                Ok('\x0c')
            }
            Some('n') => {
                self.pos += 1;
                Ok('\n')
            }
            Some('r') => {
                self.pos += 1;
                Ok('\r')
            }
            Some('t') => {
                self.pos += 1;
                Ok('\t')
            }
            _ => Err("Invalid escape sequence".to_string()),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.consume('[')?;
        self.skip_whitespace();

        let mut items = Vec::new();

        if self.peek() != Some(']') {
            loop {
                let value = self.parse_value()?;
                items.push(value);
                self.skip_whitespace();

                if self.peek() == Some(']') {
                    break;
                }
                self.consume(',')?;
                self.skip_whitespace();
            }
        }

        self.consume(']')?;
        Ok(JsonValue::Array(items))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.consume('{')?;
        self.skip_whitespace();

        let mut map = HashMap::new();

        if self.peek() != Some('}') {
            loop {
                let key = match self.parse_value()? {
                    JsonValue::String(s) => s,
                    _ => return Err("Object keys must be strings".to_string()),
                };

                self.skip_whitespace();
                self.consume(':')?;
                self.skip_whitespace();

                let value = self.parse_value()?;
                map.insert(key, value);

                self.skip_whitespace();
                if self.peek() == Some('}') {
                    break;
                }
                self.consume(',')?;
                self.skip_whitespace();
            }
        }

        self.consume('}')?;
        Ok(JsonValue::Object(map))
    }
}

pub fn pretty_print(value: &JsonValue, indent: usize) -> String {
    match value {
        JsonValue::Null => "null".to_string(),
        JsonValue::Bool(b) => b.to_string(),
        JsonValue::Number(n) => n.to_string(),
        JsonValue::String(s) => format!("\"{}\"", s),
        JsonValue::Array(arr) => {
            if arr.is_empty() {
                "[]".to_string()
            } else {
                let items: Vec<String> = arr
                    .iter()
                    .map(|item| format!("{}{}", " ".repeat(indent + 2), pretty_print(item, indent + 2)))
                    .collect();
                format!("[\n{}\n{}]", items.join(",\n"), " ".repeat(indent))
            }
        }
        JsonValue::Object(obj) => {
            if obj.is_empty() {
                "{}".to_string()
            } else {
                let items: Vec<String> = obj
                    .iter()
                    .map(|(key, value)| {
                        format!(
                            "{}\"{}\": {}",
                            " ".repeat(indent + 2),
                            key,
                            pretty_print(value, indent + 2)
                        )
                    })
                    .collect();
                format!("{{\n{}\n{}}}", items.join(",\n"), " ".repeat(indent))
            }
        }
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
    }

    #[test]
    fn test_parse_string() {
        let mut parser = JsonParser::new("\"hello\"");
        assert_eq!(
            parser.parse(),
            Ok(JsonValue::String("hello".to_string()))
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
    }

    #[test]
    fn test_parse_object() {
        let mut parser = JsonParser::new("{\"key\": \"value\"}");
        let mut map = HashMap::new();
        map.insert("key".to_string(), JsonValue::String("value".to_string()));
        assert_eq!(parser.parse(), Ok(JsonValue::Object(map)));
    }

    #[test]
    fn test_pretty_print() {
        let json = JsonValue::Object({
            let mut map = HashMap::new();
            map.insert("name".to_string(), JsonValue::String("John".to_string()));
            map.insert("age".to_string(), JsonValue::Number(30.0));
            map.insert("active".to_string(), JsonValue::Bool(true));
            map.insert("tags".to_string(), JsonValue::Array(vec![
                JsonValue::String("rust".to_string()),
                JsonValue::String("json".to_string()),
            ]));
            map
        });

        let printed = pretty_print(&json, 0);
        assert!(printed.contains("\"name\": \"John\""));
        assert!(printed.contains("\"age\": 30"));
        assert!(printed.contains("\"active\": true"));
        assert!(printed.contains("\"tags\":"));
    }
}