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

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn consume(&mut self, expected: char) -> Result<(), String> {
        self.skip_whitespace();
        if let Some(ch) = self.peek() {
            if ch == expected {
                self.pos += 1;
                Ok(())
            } else {
                Err(format!("Expected '{}', found '{}'", expected, ch))
            }
        } else {
            Err("Unexpected end of input".to_string())
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.consume('"')?;
        let mut result = String::new();
        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.pos += 1;
                return Ok(JsonValue::String(result));
            } else if ch == '\\' {
                self.pos += 1;
                if let Some(escaped) = self.peek() {
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
                    self.pos += 1;
                } else {
                    return Err("Unterminated escape sequence".to_string());
                }
            } else {
                result.push(ch);
                self.pos += 1;
            }
        }
        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '.' || ch == '-' || ch == '+' || ch == 'e' || ch == 'E' {
                self.pos += 1;
            } else {
                break;
            }
        }
        let num_str: String = self.input[start..self.pos].iter().collect();
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(format!("Invalid number: {}", num_str)),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.consume('[')?;
        self.skip_whitespace();
        let mut array = Vec::new();
        if let Some(ch) = self.peek() {
            if ch == ']' {
                self.pos += 1;
                return Ok(JsonValue::Array(array));
            }
        }
        loop {
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();
            if let Some(ch) = self.peek() {
                if ch == ']' {
                    self.pos += 1;
                    break;
                } else if ch == ',' {
                    self.pos += 1;
                    self.skip_whitespace();
                } else {
                    return Err(format!("Expected ',' or ']', found '{}'", ch));
                }
            } else {
                return Err("Unterminated array".to_string());
            }
        }
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.consume('{')?;
        self.skip_whitespace();
        let mut object = HashMap::new();
        if let Some(ch) = self.peek() {
            if ch == '}' {
                self.pos += 1;
                return Ok(JsonValue::Object(object));
            }
        }
        loop {
            let key = match self.parse_value()? {
                JsonValue::String(s) => s,
                _ => return Err("Object key must be a string".to_string()),
            };
            self.skip_whitespace();
            self.consume(':')?;
            let value = self.parse_value()?;
            object.insert(key, value);
            self.skip_whitespace();
            if let Some(ch) = self.peek() {
                if ch == '}' {
                    self.pos += 1;
                    break;
                } else if ch == ',' {
                    self.pos += 1;
                    self.skip_whitespace();
                } else {
                    return Err(format!("Expected ',' or '}}', found '{}'", ch));
                }
            } else {
                return Err("Unterminated object".to_string());
            }
        }
        Ok(JsonValue::Object(object))
    }

    fn parse_keyword(&mut self, keyword: &str, value: JsonValue) -> Result<JsonValue, String> {
        for expected in keyword.chars() {
            if let Some(ch) = self.peek() {
                if ch == expected {
                    self.pos += 1;
                } else {
                    return Err(format!("Expected '{}', found '{}'", expected, ch));
                }
            } else {
                return Err("Unexpected end of input".to_string());
            }
        }
        Ok(value)
    }

    pub fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        if let Some(ch) = self.peek() {
            match ch {
                '"' => self.parse_string(),
                '[' => self.parse_array(),
                '{' => self.parse_object(),
                't' => self.parse_keyword("true", JsonValue::Bool(true)),
                'f' => self.parse_keyword("false", JsonValue::Bool(false)),
                'n' => self.parse_keyword("null", JsonValue::Null),
                '-' | '0'..='9' => self.parse_number(),
                _ => Err(format!("Unexpected character: {}", ch)),
            }
        } else {
            Err("Unexpected end of input".to_string())
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

pub fn pretty_print_json(value: &JsonValue, indent: usize) -> String {
    match value {
        JsonValue::Null => "null".to_string(),
        JsonValue::Bool(b) => b.to_string(),
        JsonValue::Number(n) => n.to_string(),
        JsonValue::String(s) => format!("\"{}\"", s.escape_default()),
        JsonValue::Array(arr) => {
            if arr.is_empty() {
                "[]".to_string()
            } else {
                let items: Vec<String> = arr
                    .iter()
                    .map(|item| {
                        let mut result = " ".repeat(indent + 2);
                        result.push_str(&pretty_print_json(item, indent + 2));
                        result
                    })
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
                        let mut result = " ".repeat(indent + 2);
                        result.push_str(&format!("\"{}\": ", key.escape_default()));
                        result.push_str(&pretty_print_json(value, indent + 2));
                        result
                    })
                    .collect();
                format!("{{\n{}\n{}}}", items.join(",\n"), " ".repeat(indent))
            }
        }
    }
}