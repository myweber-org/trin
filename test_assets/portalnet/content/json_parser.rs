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

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        if self.position >= self.input.len() {
            return Err("Unexpected end of input".to_string());
        }

        match self.input[self.position] {
            'n' => self.parse_null(),
            't' | 'f' => self.parse_bool(),
            '"' => self.parse_string(),
            '[' => self.parse_array(),
            '{' => self.parse_object(),
            '-' | '0'..='9' => self.parse_number(),
            _ => Err(format!("Unexpected character: {}", self.input[self.position])),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        if self.position + 3 < self.input.len()
            && self.input[self.position..self.position + 4]
                .iter()
                .collect::<String>()
                == "null"
        {
            self.position += 4;
            Ok(JsonValue::Null)
        } else {
            Err("Expected 'null'".to_string())
        }
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.position + 3 < self.input.len()
            && self.input[self.position..self.position + 4]
                .iter()
                .collect::<String>()
                == "true"
        {
            self.position += 4;
            Ok(JsonValue::Bool(true))
        } else if self.position + 4 < self.input.len()
            && self.input[self.position..self.position + 5]
                .iter()
                .collect::<String>()
                == "false"
        {
            self.position += 5;
            Ok(JsonValue::Bool(false))
        } else {
            Err("Expected 'true' or 'false'".to_string())
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.position += 1;
        let mut result = String::new();

        while self.position < self.input.len() && self.input[self.position] != '"' {
            result.push(self.input[self.position]);
            self.position += 1;
        }

        if self.position < self.input.len() && self.input[self.position] == '"' {
            self.position += 1;
            Ok(JsonValue::String(result))
        } else {
            Err("Unterminated string".to_string())
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.position;
        while self.position < self.input.len()
            && (self.input[self.position].is_ascii_digit()
                || self.input[self.position] == '.'
                || self.input[self.position] == '-'
                || self.input[self.position] == 'e'
                || self.input[self.position] == 'E')
        {
            self.position += 1;
        }

        let num_str: String = self.input[start..self.position].iter().collect();
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(format!("Invalid number: {}", num_str)),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.position += 1;
        self.skip_whitespace();
        let mut array = Vec::new();

        if self.position < self.input.len() && self.input[self.position] == ']' {
            self.position += 1;
            return Ok(JsonValue::Array(array));
        }

        loop {
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();

            if self.position < self.input.len() && self.input[self.position] == ',' {
                self.position += 1;
                self.skip_whitespace();
            } else if self.position < self.input.len() && self.input[self.position] == ']' {
                self.position += 1;
                break;
            } else {
                return Err("Expected ',' or ']' in array".to_string());
            }
        }

        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.position += 1;
        self.skip_whitespace();
        let mut object = HashMap::new();

        if self.position < self.input.len() && self.input[self.position] == '}' {
            self.position += 1;
            return Ok(JsonValue::Object(object));
        }

        loop {
            self.skip_whitespace();
            if self.position >= self.input.len() || self.input[self.position] != '"' {
                return Err("Expected string key in object".to_string());
            }

            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => return Err("Expected string key".to_string()),
            };

            self.skip_whitespace();
            if self.position >= self.input.len() || self.input[self.position] != ':' {
                return Err("Expected ':' after key".to_string());
            }
            self.position += 1;

            let value = self.parse_value()?;
            object.insert(key, value);
            self.skip_whitespace();

            if self.position < self.input.len() && self.input[self.position] == ',' {
                self.position += 1;
                self.skip_whitespace();
            } else if self.position < self.input.len() && self.input[self.position] == '}' {
                self.position += 1;
                break;
            } else {
                return Err("Expected ',' or '}' in object".to_string());
            }
        }

        Ok(JsonValue::Object(object))
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.position < self.input.len() {
            Err("Extra characters after JSON value".to_string())
        } else {
            Ok(result)
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
    fn test_parse_string() {
        let mut parser = JsonParser::new(r#""hello world""#);
        assert_eq!(
            parser.parse(),
            Ok(JsonValue::String("hello world".to_string()))
        );
    }

    #[test]
    fn test_parse_number() {
        let mut parser = JsonParser::new("42.5");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(42.5)));
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
        let mut parser = JsonParser::new(r#"{"key": "value"}"#);
        let mut expected = HashMap::new();
        expected.insert("key".to_string(), JsonValue::String("value".to_string()));
        assert_eq!(parser.parse(), Ok(JsonValue::Object(expected)));
    }
}