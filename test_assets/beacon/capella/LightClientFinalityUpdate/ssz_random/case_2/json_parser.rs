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

struct JsonParser {
    input: Vec<char>,
    pos: usize,
}

impl JsonParser {
    fn new(input: &str) -> Self {
        JsonParser {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    fn parse(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            return Err("Unexpected trailing characters".to_string());
        }
        Ok(result)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        match self.peek() {
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
            Err("Expected boolean".to_string())
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        if self.peek() == Some('-') {
            self.consume();
        }
        while let Some(c) = self.peek() {
            if c.is_digit(10) {
                self.consume();
            } else {
                break;
            }
        }
        if self.peek() == Some('.') {
            self.consume();
            while let Some(c) = self.peek() {
                if c.is_digit(10) {
                    self.consume();
                } else {
                    break;
                }
            }
        }
        let num_str: String = self.input[start..self.pos].iter().collect();
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err("Invalid number".to_string()),
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.expect('"')?;
        let mut result = String::new();
        while let Some(c) = self.consume() {
            if c == '"' {
                break;
            }
            result.push(c);
        }
        Ok(JsonValue::String(result))
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.expect('[')?;
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
                self.expect(',')?;
                self.skip_whitespace();
            }
        }
        self.expect(']')?;
        Ok(JsonValue::Array(items))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.expect('{')?;
        self.skip_whitespace();
        let mut map = HashMap::new();
        if self.peek() != Some('}') {
            loop {
                let key = match self.parse_string()? {
                    JsonValue::String(s) => s,
                    _ => return Err("Expected string key".to_string()),
                };
                self.skip_whitespace();
                self.expect(':')?;
                self.skip_whitespace();
                let value = self.parse_value()?;
                map.insert(key, value);
                self.skip_whitespace();
                if self.peek() == Some('}') {
                    break;
                }
                self.expect(',')?;
                self.skip_whitespace();
            }
        }
        self.expect('}')?;
        Ok(JsonValue::Object(map))
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.consume();
            } else {
                break;
            }
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn consume(&mut self) -> Option<char> {
        let c = self.peek();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn expect(&mut self, expected: char) -> Result<(), String> {
        match self.consume() {
            Some(c) if c == expected => Ok(()),
            Some(c) => Err(format!("Expected '{}', found '{}'", expected, c)),
            None => Err(format!("Expected '{}', found EOF", expected)),
        }
    }

    fn consume_str(&mut self, s: &str) -> bool {
        let old_pos = self.pos;
        for expected in s.chars() {
            if self.consume() != Some(expected) {
                self.pos = old_pos;
                return false;
            }
        }
        true
    }
}

fn pretty_print_json(value: &JsonValue, indent: usize) -> String {
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
                    .map(|item| format!("{}{}", " ".repeat(indent + 2), pretty_print_json(item, indent + 2)))
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
                            pretty_print_json(value, indent + 2)
                        )
                    })
                    .collect();
                format!("{{\n{}\n{}}}", items.join(",\n"), " ".repeat(indent))
            }
        }
    }
}

fn main() {
    let json_str = r#"
    {
        "name": "John",
        "age": 30,
        "is_student": false,
        "courses": ["Math", "Science"],
        "address": {
            "city": "New York",
            "zip": "10001"
        }
    }
    "#;

    let mut parser = JsonParser::new(json_str);
    match parser.parse() {
        Ok(json) => {
            println!("Parsed successfully!");
            println!("Pretty printed:\n{}", pretty_print_json(&json, 0));
        }
        Err(e) => println!("Parse error: {}", e),
    }
}