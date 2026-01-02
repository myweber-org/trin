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

impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonValue::Null => write!(f, "null"),
            JsonValue::Bool(b) => write!(f, "{}", b),
            JsonValue::Number(n) => write!(f, "{}", n),
            JsonValue::String(s) => write!(f, "\"{}\"", s),
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
                    write!(f, "\"{}\": {}", key, value)?;
                }
                write!(f, "}}")
            }
        }
    }
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
        let mut result = String::new();
        let mut escape = false;

        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            self.pos += 1;

            if escape {
                match c {
                    '"' => result.push('"'),
                    '\\' => result.push('\\'),
                    '/' => result.push('/'),
                    'b' => result.push('\u{0008}'),
                    'f' => result.push('\u{000C}'),
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    _ => return Err(format!("Invalid escape sequence: \\{}", c)),
                }
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                return Ok(JsonValue::String(result));
            } else {
                result.push(c);
            }
        }

        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        let mut has_dot = false;

        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            if c == '.' {
                if has_dot {
                    return Err("Invalid number format".to_string());
                }
                has_dot = true;
                self.pos += 1;
            } else if c.is_ascii_digit() || (self.pos == start && c == '-') {
                self.pos += 1;
            } else {
                break;
            }
        }

        let num_str = &self.input[start..self.pos];
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err("Invalid number format".to_string()),
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
                return Err("Expected ':' after object key".to_string());
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

pub fn pretty_print(json: &JsonValue, indent: usize) -> String {
    fn pretty_print_inner(json: &JsonValue, indent: usize, current_indent: usize) -> String {
        let indent_str = " ".repeat(current_indent);
        match json {
            JsonValue::Null => "null".to_string(),
            JsonValue::Bool(b) => b.to_string(),
            JsonValue::Number(n) => n.to_string(),
            JsonValue::String(s) => format!("\"{}\"", s),
            JsonValue::Array(arr) => {
                if arr.is_empty() {
                    return "[]".to_string();
                }
                let mut result = "[\n".to_string();
                for (i, item) in arr.iter().enumerate() {
                    result.push_str(&format!(
                        "{}{}",
                        " ".repeat(current_indent + indent),
                        pretty_print_inner(item, indent, current_indent + indent)
                    ));
                    if i < arr.len() - 1 {
                        result.push_str(",\n");
                    } else {
                        result.push('\n');
                    }
                }
                result.push_str(&format!("{}{}", indent_str, "]"));
                result
            }
            JsonValue::Object(obj) => {
                if obj.is_empty() {
                    return "{}".to_string();
                }
                let mut result = "{\n".to_string();
                let keys: Vec<&String> = obj.keys().collect();
                for (i, key) in keys.iter().enumerate() {
                    let value = obj.get(*key).unwrap();
                    result.push_str(&format!(
                        "{}\"{}\": {}",
                        " ".repeat(current_indent + indent),
                        key,
                        pretty_print_inner(value, indent, current_indent + indent)
                    ));
                    if i < keys.len() - 1 {
                        result.push_str(",\n");
                    } else {
                        result.push('\n');
                    }
                }
                result.push_str(&format!("{}{}", indent_str, "}"));
                result
            }
        }
    }
    pretty_print_inner(json, indent, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_json() {
        let json_str = r#"{"name": "John", "age": 30, "active": true}"#;
        let mut parser = JsonParser::new(json_str);
        let result = parser.parse();
        assert!(result.is_ok());
        
        if let Ok(JsonValue::Object(obj)) = result {
            assert_eq!(obj.get("name"), Some(&JsonValue::String("John".to_string())));
            assert_eq!(obj.get("age"), Some(&JsonValue::Number(30.0)));
            assert_eq!(obj.get("active"), Some(&JsonValue::Bool(true)));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_parse_array() {
        let json_str = r#"[1, 2, 3, "four", true, null]"#;
        let mut parser = JsonParser::new(json_str);
        let result = parser.parse();
        assert!(result.is_ok());
        
        if let Ok(JsonValue::Array(arr)) = result {
            assert_eq!(arr.len(), 6);
            assert_eq!(arr[0], JsonValue::Number(1.0));
            assert_eq!(arr[3], JsonValue::String("four".to_string()));
            assert_eq!(arr[5], JsonValue::Null);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_pretty_print() {
        let json = JsonValue::Object({
            let mut map = HashMap::new();
            map.insert("name".to_string(), JsonValue::String("Alice".to_string()));
            map.insert("scores".to_string(), JsonValue::Array(vec![
                JsonValue::Number(95.5),
                JsonValue::Number(88.0),
                JsonValue::Number(92.0),
            ]));
            map
        });

        let printed = pretty_print(&json, 2);
        assert!(printed.contains("\"name\": \"Alice\""));
        assert!(printed.contains("\"scores\": ["));
    }
}