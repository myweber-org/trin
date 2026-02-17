use std::collections::HashMap;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
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

    fn peek(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn consume(&mut self, expected: char) -> Result<(), String> {
        self.skip_whitespace();
        if let Some(ch) = self.peek() {
            if ch == expected {
                self.position += 1;
                return Ok(());
            }
        }
        Err(format!("Expected '{}' at position {}", expected, self.position))
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.consume('"')?;
        let mut result = String::new();
        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.position += 1;
                return Ok(result);
            }
            if ch == '\\' {
                self.position += 1;
                let escaped = self.peek().ok_or("Unexpected end of string after escape")?;
                result.push(match escaped {
                    '"' => '"',
                    '\\' => '\\',
                    '/' => '/',
                    'b' => '\u{0008}',
                    'f' => '\u{000c}',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    _ => return Err(format!("Invalid escape sequence: \\{}", escaped)),
                });
                self.position += 1;
            } else {
                result.push(ch);
                self.position += 1;
            }
        }
        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<f64, String> {
        let start = self.position;
        while let Some(ch) = self.peek() {
            if ch.is_digit(10) || ch == '.' || ch == '-' || ch == '+' || ch == 'e' || ch == 'E' {
                self.position += 1;
            } else {
                break;
            }
        }
        let num_str: String = self.input[start..self.position].iter().collect();
        num_str.parse::<f64>().map_err(|e| format!("Invalid number: {}", e))
    }

    fn parse_array(&mut self) -> Result<Vec<JsonValue>, String> {
        self.consume('[')?;
        self.skip_whitespace();
        let mut array = Vec::new();
        if let Some(ch) = self.peek() {
            if ch == ']' {
                self.position += 1;
                return Ok(array);
            }
        }
        loop {
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();
            if let Some(ch) = self.peek() {
                if ch == ']' {
                    self.position += 1;
                    break;
                } else if ch == ',' {
                    self.position += 1;
                    self.skip_whitespace();
                } else {
                    return Err(format!("Expected ',' or ']' at position {}", self.position));
                }
            } else {
                return Err("Unexpected end of array".to_string());
            }
        }
        Ok(array)
    }

    fn parse_object(&mut self) -> Result<HashMap<String, JsonValue>, String> {
        self.consume('{')?;
        self.skip_whitespace();
        let mut map = HashMap::new();
        if let Some(ch) = self.peek() {
            if ch == '}' {
                self.position += 1;
                return Ok(map);
            }
        }
        loop {
            let key = self.parse_string()?;
            self.skip_whitespace();
            self.consume(':')?;
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();
            if let Some(ch) = self.peek() {
                if ch == '}' {
                    self.position += 1;
                    break;
                } else if ch == ',' {
                    self.position += 1;
                    self.skip_whitespace();
                } else {
                    return Err(format!("Expected ',' or '}}' at position {}", self.position));
                }
            } else {
                return Err("Unexpected end of object".to_string());
            }
        }
        Ok(map)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        let ch = self.peek().ok_or("Unexpected end of input")?;
        match ch {
            'n' => {
                if self.input[self.position..].starts_with(&['n', 'u', 'l', 'l']) {
                    self.position += 4;
                    Ok(JsonValue::Null)
                } else {
                    Err("Expected 'null'".to_string())
                }
            }
            't' => {
                if self.input[self.position..].starts_with(&['t', 'r', 'u', 'e']) {
                    self.position += 4;
                    Ok(JsonValue::Bool(true))
                } else {
                    Err("Expected 'true'".to_string())
                }
            }
            'f' => {
                if self.input[self.position..].starts_with(&['f', 'a', 'l', 's', 'e']) {
                    self.position += 5;
                    Ok(JsonValue::Bool(false))
                } else {
                    Err("Expected 'false'".to_string())
                }
            }
            '"' => {
                let s = self.parse_string()?;
                Ok(JsonValue::String(s))
            }
            '[' => {
                let arr = self.parse_array()?;
                Ok(JsonValue::Array(arr))
            }
            '{' => {
                let obj = self.parse_object()?;
                Ok(JsonValue::Object(obj))
            }
            '-' | '0'..='9' => {
                let num = self.parse_number()?;
                Ok(JsonValue::Number(num))
            }
            _ => Err(format!("Unexpected character '{}' at position {}", ch, self.position)),
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.position < self.input.len() {
            return Err("Trailing characters after JSON value".to_string());
        }
        Ok(result)
    }
}

impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonValue::Null => write!(f, "null"),
            JsonValue::Bool(b) => write!(f, "{}", b),
            JsonValue::Number(n) => write!(f, "{}", n),
            JsonValue::String(s) => write!(f, "\"{}\"", s.escape_default()),
            JsonValue::Array(arr) => {
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            JsonValue::Object(obj) => {
                write!(f, "{{")?;
                for (i, (key, value)) in obj.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", key.escape_default(), value)?;
                }
                write!(f, "}}")
            }
        }
    }
}

pub fn pretty_print_json(value: &JsonValue, indent: usize) -> String {
    fn pretty_print(value: &JsonValue, indent: usize, level: usize) -> String {
        let spaces = " ".repeat(level * indent);
        match value {
            JsonValue::Null => "null".to_string(),
            JsonValue::Bool(b) => b.to_string(),
            JsonValue::Number(n) => n.to_string(),
            JsonValue::String(s) => format!("\"{}\"", s.escape_default()),
            JsonValue::Array(arr) => {
                if arr.is_empty() {
                    return "[]".to_string();
                }
                let mut result = "[\n".to_string();
                for (i, item) in arr.iter().enumerate() {
                    result.push_str(&format!("{}{}", spaces, " ".repeat(indent)));
                    result.push_str(&pretty_print(item, indent, level + 1));
                    if i < arr.len() - 1 {
                        result.push(',');
                    }
                    result.push('\n');
                }
                result.push_str(&format!("{}{}", spaces, "]"));
                result
            }
            JsonValue::Object(obj) => {
                if obj.is_empty() {
                    return "{}".to_string();
                }
                let mut result = "{\n".to_string();
                let keys: Vec<&String> = obj.keys().collect();
                for (i, key) in keys.iter().enumerate() {
                    result.push_str(&format!("{}{}", spaces, " ".repeat(indent)));
                    result.push_str(&format!("\"{}\": ", key.escape_default()));
                    result.push_str(&pretty_print(&obj[*key], indent, level + 1));
                    if i < keys.len() - 1 {
                        result.push(',');
                    }
                    result.push('\n');
                }
                result.push_str(&format!("{}{}", spaces, "}"));
                result
            }
        }
    }
    pretty_print(value, indent, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_json() {
        let json = r#"{"name": "John", "age": 30, "active": true}"#;
        let mut parser = JsonParser::new(json);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_array() {
        let json = r#"[1, 2, 3, "four", false]"#;
        let mut parser = JsonParser::new(json);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_pretty_print() {
        let json = r#"{"name":"John","age":30,"hobbies":["reading","coding"]}"#;
        let mut parser = JsonParser::new(json);
        let value = parser.parse().unwrap();
        let pretty = pretty_print_json(&value, 2);
        assert!(pretty.contains('\n'));
        assert!(pretty.contains("  "));
    }
}