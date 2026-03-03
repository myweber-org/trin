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

#[derive(Debug, PartialEq)]
enum Token {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Colon,
    Comma,
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}

struct JsonParser {
    input: String,
    position: usize,
}

impl JsonParser {
    fn new(input: &str) -> Self {
        JsonParser {
            input: input.to_string(),
            position: 0,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() {
            let c = self.input.chars().nth(self.position).unwrap();
            if c.is_whitespace() {
                self.position += 1;
            } else {
                break;
            }
        }
    }

    fn parse_string(&mut self) -> Option<Token> {
        if self.input.chars().nth(self.position)? != '"' {
            return None;
        }
        self.position += 1;
        let start = self.position;
        while self.position < self.input.len() {
            let c = self.input.chars().nth(self.position).unwrap();
            if c == '"' {
                let s = self.input[start..self.position].to_string();
                self.position += 1;
                return Some(Token::String(s));
            }
            self.position += 1;
        }
        None
    }

    fn parse_number(&mut self) -> Option<Token> {
        let start = self.position;
        while self.position < self.input.len() {
            let c = self.input.chars().nth(self.position).unwrap();
            if c.is_digit(10) || c == '.' || c == '-' || c == 'e' || c == 'E' {
                self.position += 1;
            } else {
                break;
            }
        }
        if start == self.position {
            return None;
        }
        let num_str = &self.input[start..self.position];
        match num_str.parse::<f64>() {
            Ok(num) => Some(Token::Number(num)),
            Err(_) => None,
        }
    }

    fn parse_keyword(&mut self, keyword: &str, token: Token) -> Option<Token> {
        if self.position + keyword.len() <= self.input.len() {
            let slice = &self.input[self.position..self.position + keyword.len()];
            if slice == keyword {
                self.position += keyword.len();
                return Some(token);
            }
        }
        None
    }

    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        if self.position >= self.input.len() {
            return None;
        }

        let c = self.input.chars().nth(self.position).unwrap();
        match c {
            '{' => {
                self.position += 1;
                Some(Token::LeftBrace)
            }
            '}' => {
                self.position += 1;
                Some(Token::RightBrace)
            }
            '[' => {
                self.position += 1;
                Some(Token::LeftBracket)
            }
            ']' => {
                self.position += 1;
                Some(Token::RightBracket)
            }
            ':' => {
                self.position += 1;
                Some(Token::Colon)
            }
            ',' => {
                self.position += 1;
                Some(Token::Comma)
            }
            '"' => self.parse_string(),
            '0'..='9' | '-' => self.parse_number(),
            't' => self.parse_keyword("true", Token::Bool(true)),
            'f' => self.parse_keyword("false", Token::Bool(false)),
            'n' => self.parse_keyword("null", Token::Null),
            _ => None,
        }
    }

    fn parse_value(&mut self) -> Option<JsonValue> {
        self.skip_whitespace();
        if self.position >= self.input.len() {
            return None;
        }

        let c = self.input.chars().nth(self.position).unwrap();
        match c {
            '{' => self.parse_object(),
            '[' => self.parse_array(),
            '"' => {
                if let Some(Token::String(s)) = self.parse_string() {
                    Some(JsonValue::String(s))
                } else {
                    None
                }
            }
            '0'..='9' | '-' => {
                if let Some(Token::Number(n)) = self.parse_number() {
                    Some(JsonValue::Number(n))
                } else {
                    None
                }
            }
            't' | 'f' => {
                if let Some(Token::Bool(b)) = self.parse_keyword(
                    if c == 't' { "true" } else { "false" },
                    Token::Bool(c == 't'),
                ) {
                    Some(JsonValue::Bool(b))
                } else {
                    None
                }
            }
            'n' => {
                if let Some(Token::Null) = self.parse_keyword("null", Token::Null) {
                    Some(JsonValue::Null)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn parse_object(&mut self) -> Option<JsonValue> {
        if self.input.chars().nth(self.position)? != '{' {
            return None;
        }
        self.position += 1;

        let mut map = HashMap::new();
        self.skip_whitespace();

        if self.input.chars().nth(self.position)? == '}' {
            self.position += 1;
            return Some(JsonValue::Object(map));
        }

        loop {
            self.skip_whitespace();
            let key = if let Some(Token::String(s)) = self.parse_string() {
                s
            } else {
                return None;
            };

            self.skip_whitespace();
            if self.input.chars().nth(self.position)? != ':' {
                return None;
            }
            self.position += 1;

            let value = self.parse_value()?;
            map.insert(key, value);

            self.skip_whitespace();
            if self.position >= self.input.len() {
                return None;
            }

            let c = self.input.chars().nth(self.position).unwrap();
            if c == '}' {
                self.position += 1;
                break;
            } else if c == ',' {
                self.position += 1;
                continue;
            } else {
                return None;
            }
        }

        Some(JsonValue::Object(map))
    }

    fn parse_array(&mut self) -> Option<JsonValue> {
        if self.input.chars().nth(self.position)? != '[' {
            return None;
        }
        self.position += 1;

        let mut arr = Vec::new();
        self.skip_whitespace();

        if self.input.chars().nth(self.position)? == ']' {
            self.position += 1;
            return Some(JsonValue::Array(arr));
        }

        loop {
            let value = self.parse_value()?;
            arr.push(value);

            self.skip_whitespace();
            if self.position >= self.input.len() {
                return None;
            }

            let c = self.input.chars().nth(self.position).unwrap();
            if c == ']' {
                self.position += 1;
                break;
            } else if c == ',' {
                self.position += 1;
                continue;
            } else {
                return None;
            }
        }

        Some(JsonValue::Array(arr))
    }

    fn parse(&mut self) -> Option<JsonValue> {
        self.parse_value()
    }
}

fn parse_json(input: &str) -> Option<JsonValue> {
    let mut parser = JsonParser::new(input);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_object() {
        let json = r#"{"name": "test", "value": 42}"#;
        let result = parse_json(json);
        assert!(result.is_some());
        if let Some(JsonValue::Object(map)) = result {
            assert_eq!(map.len(), 2);
            if let Some(JsonValue::String(name)) = map.get("name") {
                assert_eq!(name, "test");
            } else {
                panic!("Expected string for key 'name'");
            }
            if let Some(JsonValue::Number(value)) = map.get("value") {
                assert_eq!(*value, 42.0);
            } else {
                panic!("Expected number for key 'value'");
            }
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_parse_array() {
        let json = r#"[1, 2, 3, "four"]"#;
        let result = parse_json(json);
        assert!(result.is_some());
        if let Some(JsonValue::Array(arr)) = result {
            assert_eq!(arr.len(), 4);
            assert_eq!(arr[0], JsonValue::Number(1.0));
            assert_eq!(arr[1], JsonValue::Number(2.0));
            assert_eq!(arr[2], JsonValue::Number(3.0));
            assert_eq!(arr[3], JsonValue::String("four".to_string()));
        } else {
            panic!("Expected array");
        }
    }
}