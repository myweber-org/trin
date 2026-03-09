use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
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

struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && self.input[self.position].is_whitespace() {
            self.position += 1;
        }
    }

    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        if self.position >= self.input.len() {
            return None;
        }

        let ch = self.input[self.position];
        match ch {
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
            't' | 'f' | 'n' => self.parse_keyword(),
            _ => None,
        }
    }

    fn parse_string(&mut self) -> Option<Token> {
        self.position += 1;
        let start = self.position;
        while self.position < self.input.len() && self.input[self.position] != '"' {
            self.position += 1;
        }
        let result: String = self.input[start..self.position].iter().collect();
        self.position += 1;
        Some(Token::String(result))
    }

    fn parse_number(&mut self) -> Option<Token> {
        let start = self.position;
        while self.position < self.input.len() && (self.input[self.position].is_digit(10) || self.input[self.position] == '.' || self.input[self.position] == '-') {
            self.position += 1;
        }
        let num_str: String = self.input[start..self.position].iter().collect();
        match num_str.parse::<f64>() {
            Ok(num) => Some(Token::Number(num)),
            Err(_) => None,
        }
    }

    fn parse_keyword(&mut self) -> Option<Token> {
        let start = self.position;
        while self.position < self.input.len() && self.input[self.position].is_alphabetic() {
            self.position += 1;
        }
        let keyword: String = self.input[start..self.position].iter().collect();
        match keyword.as_str() {
            "true" => Some(Token::Bool(true)),
            "false" => Some(Token::Bool(false)),
            "null" => Some(Token::Null),
            _ => None,
        }
    }
}

struct Parser {
    lexer: Lexer,
    current_token: Option<Token>,
}

impl Parser {
    fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token();
        Parser { lexer, current_token }
    }

    fn parse(&mut self) -> Option<JsonValue> {
        self.parse_value()
    }

    fn parse_value(&mut self) -> Option<JsonValue> {
        match &self.current_token {
            Some(Token::Null) => {
                self.advance();
                Some(JsonValue::Null)
            }
            Some(Token::Bool(b)) => {
                let value = *b;
                self.advance();
                Some(JsonValue::Bool(value))
            }
            Some(Token::Number(n)) => {
                let value = *n;
                self.advance();
                Some(JsonValue::Number(value))
            }
            Some(Token::String(s)) => {
                let value = s.clone();
                self.advance();
                Some(JsonValue::String(value))
            }
            Some(Token::LeftBracket) => self.parse_array(),
            Some(Token::LeftBrace) => self.parse_object(),
            _ => None,
        }
    }

    fn parse_array(&mut self) -> Option<JsonValue> {
        self.advance();
        let mut array = Vec::new();

        while let Some(token) = &self.current_token {
            if *token == Token::RightBracket {
                self.advance();
                return Some(JsonValue::Array(array));
            }

            if let Some(value) = self.parse_value() {
                array.push(value);
            }

            if let Some(Token::Comma) = &self.current_token {
                self.advance();
            } else if self.current_token != Some(Token::RightBracket) {
                return None;
            }
        }
        None
    }

    fn parse_object(&mut self) -> Option<JsonValue> {
        self.advance();
        let mut map = HashMap::new();

        while let Some(token) = &self.current_token {
            if *token == Token::RightBrace {
                self.advance();
                return Some(JsonValue::Object(map));
            }

            let key = match token {
                Token::String(s) => s.clone(),
                _ => return None,
            };
            self.advance();

            if self.current_token != Some(Token::Colon) {
                return None;
            }
            self.advance();

            if let Some(value) = self.parse_value() {
                map.insert(key, value);
            }

            if let Some(Token::Comma) = &self.current_token {
                self.advance();
            } else if self.current_token != Some(Token::RightBrace) {
                return None;
            }
        }
        None
    }

    fn advance(&mut self) {
        self.current_token = self.lexer.next_token();
    }
}

fn parse_json(input: &str) -> Option<JsonValue> {
    let mut parser = Parser::new(input);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_object() {
        let input = r#"{"name": "test", "value": 42}"#;
        let result = parse_json(input);
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_array() {
        let input = r#"[1, 2, 3, "hello"]"#;
        let result = parse_json(input);
        assert!(result.is_some());
    }
}