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
        let value = self.parse_value()?;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            return Err("Unexpected trailing characters".to_string());
        }
        Ok(value)
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
        if self.consume("null") {
            Ok(JsonValue::Null)
        } else {
            Err("Expected 'null'".to_string())
        }
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.consume("true") {
            Ok(JsonValue::Bool(true))
        } else if self.consume("false") {
            Ok(JsonValue::Bool(false))
        } else {
            Err("Expected 'true' or 'false'".to_string())
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.expect('"')?;
        let mut result = String::new();
        while let Some(c) = self.next_char() {
            match c {
                '"' => break,
                '\\' => {
                    let escaped = self.next_char().ok_or("Unexpected end of string")?;
                    result.push(match escaped {
                        '"' => '"',
                        '\\' => '\\',
                        '/' => '/',
                        'b' => '\x08',
                        'f' => '\x0c',
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        _ => return Err("Invalid escape sequence".to_string()),
                    });
                }
                _ => result.push(c),
            }
        }
        Ok(JsonValue::String(result))
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        if self.peek() == Some('-') {
            self.next_char();
        }
        while let Some(c) = self.peek() {
            if c.is_digit(10) {
                self.next_char();
            } else {
                break;
            }
        }
        if self.peek() == Some('.') {
            self.next_char();
            while let Some(c) = self.peek() {
                if c.is_digit(10) {
                    self.next_char();
                } else {
                    break;
                }
            }
        }
        if let Some('e') | Some('E') = self.peek() {
            self.next_char();
            if let Some('+') | Some('-') = self.peek() {
                self.next_char();
            }
            while let Some(c) = self.peek() {
                if c.is_digit(10) {
                    self.next_char();
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

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.expect('[')?;
        self.skip_whitespace();
        let mut array = Vec::new();
        if self.peek() != Some(']') {
            loop {
                let value = self.parse_value()?;
                array.push(value);
                self.skip_whitespace();
                if self.peek() != Some(',') {
                    break;
                }
                self.next_char();
                self.skip_whitespace();
            }
        }
        self.expect(']')?;
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.expect('{')?;
        self.skip_whitespace();
        let mut object = HashMap::new();
        if self.peek() != Some('}') {
            loop {
                let key = match self.parse_string()? {
                    JsonValue::String(s) => s,
                    _ => unreachable!(),
                };
                self.skip_whitespace();
                self.expect(':')?;
                self.skip_whitespace();
                let value = self.parse_value()?;
                object.insert(key, value);
                self.skip_whitespace();
                if self.peek() != Some(',') {
                    break;
                }
                self.next_char();
                self.skip_whitespace();
            }
        }
        self.expect('}')?;
        Ok(JsonValue::Object(object))
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn next_char(&mut self) -> Option<char> {
        let c = self.peek();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn expect(&mut self, expected: char) -> Result<(), String> {
        match self.next_char() {
            Some(c) if c == expected => Ok(()),
            Some(c) => Err(format!("Expected '{}', found '{}'", expected, c)),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    fn consume(&mut self, target: &str) -> bool {
        let target_chars: Vec<char> = target.chars().collect();
        if self.pos + target_chars.len() <= self.input.len() {
            for (i, &c) in target_chars.iter().enumerate() {
                if self.input[self.pos + i] != c {
                    return false;
                }
            }
            self.pos += target_chars.len();
            true
        } else {
            false
        }
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
        "name": "John Doe",
        "age": 30,
        "is_student": false,
        "courses": ["Math", "Science"],
        "address": {
            "street": "123 Main St",
            "city": "Anytown"
        }
    }
    "#;

    let mut parser = JsonParser::new(json_str);
    match parser.parse() {
        Ok(json_value) => {
            println!("Parsed successfully!");
            println!("{}", pretty_print_json(&json_value, 0));
        }
        Err(err) => println!("Parse error: {}", err),
    }
}