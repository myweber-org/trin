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

pub fn parse_json(input: &str) -> Result<JsonValue, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Empty input".to_string());
    }
    
    let first_char = trimmed.chars().next().unwrap();
    match first_char {
        'n' => parse_null(trimmed),
        't' | 'f' => parse_bool(trimmed),
        '"' => parse_string(trimmed),
        '[' => parse_array(trimmed),
        '{' => parse_object(trimmed),
        '0'..='9' | '-' => parse_number(trimmed),
        _ => Err(format!("Unexpected character: {}", first_char)),
    }
}

fn parse_null(input: &str) -> Result<JsonValue, String> {
    if input == "null" {
        Ok(JsonValue::Null)
    } else {
        Err("Invalid null value".to_string())
    }
}

fn parse_bool(input: &str) -> Result<JsonValue, String> {
    match input {
        "true" => Ok(JsonValue::Bool(true)),
        "false" => Ok(JsonValue::Bool(false)),
        _ => Err("Invalid boolean value".to_string()),
    }
}

fn parse_number(input: &str) -> Result<JsonValue, String> {
    match input.parse::<f64>() {
        Ok(num) => Ok(JsonValue::Number(num)),
        Err(_) => Err("Invalid number format".to_string()),
    }
}

fn parse_string(input: &str) -> Result<JsonValue, String> {
    if input.starts_with('"') && input.ends_with('"') {
        let content = &input[1..input.len()-1];
        Ok(JsonValue::String(content.to_string()))
    } else {
        Err("Invalid string format".to_string())
    }
}

fn parse_array(input: &str) -> Result<JsonValue, String> {
    if !input.starts_with('[') || !input.ends_with(']') {
        return Err("Invalid array format".to_string());
    }
    
    let content = &input[1..input.len()-1].trim();
    if content.is_empty() {
        return Ok(JsonValue::Array(Vec::new()));
    }
    
    let mut items = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    let mut in_string = false;
    
    for ch in content.chars() {
        match ch {
            '"' if !in_string => in_string = true,
            '"' if in_string => in_string = false,
            '[' | '{' if !in_string => depth += 1,
            ']' | '}' if !in_string => depth -= 1,
            ',' if !in_string && depth == 0 => {
                items.push(parse_json(&current.trim())?);
                current.clear();
                continue;
            }
            _ => {}
        }
        current.push(ch);
    }
    
    if !current.is_empty() {
        items.push(parse_json(&current.trim())?);
    }
    
    Ok(JsonValue::Array(items))
}

fn parse_object(input: &str) -> Result<JsonValue, String> {
    if !input.starts_with('{') || !input.ends_with('}') {
        return Err("Invalid object format".to_string());
    }
    
    let content = &input[1..input.len()-1].trim();
    if content.is_empty() {
        return Ok(JsonValue::Object(HashMap::new()));
    }
    
    let mut map = HashMap::new();
    let mut current_key = String::new();
    let mut current_value = String::new();
    let mut parsing_key = true;
    let mut depth = 0;
    let mut in_string = false;
    
    for ch in content.chars() {
        match ch {
            '"' if !in_string => in_string = true,
            '"' if in_string => in_string = false,
            ':' if !in_string && parsing_key && depth == 0 => {
                parsing_key = false;
                continue;
            }
            ',' if !in_string && !parsing_key && depth == 0 => {
                let key = parse_json(&current_key.trim())?;
                let value = parse_json(&current_value.trim())?;
                
                if let JsonValue::String(key_str) = key {
                    map.insert(key_str, value);
                } else {
                    return Err("Object keys must be strings".to_string());
                }
                
                current_key.clear();
                current_value.clear();
                parsing_key = true;
                continue;
            }
            '[' | '{' if !in_string => depth += 1,
            ']' | '}' if !in_string => depth -= 1,
            _ => {}
        }
        
        if parsing_key {
            current_key.push(ch);
        } else {
            current_value.push(ch);
        }
    }
    
    if !current_key.is_empty() && !current_value.is_empty() {
        let key = parse_json(&current_key.trim())?;
        let value = parse_json(&current_value.trim())?;
        
        if let JsonValue::String(key_str) = key {
            map.insert(key_str, value);
        } else {
            return Err("Object keys must be strings".to_string());
        }
    }
    
    Ok(JsonValue::Object(map))
}

pub fn pretty_print_json(value: &JsonValue, indent: usize) -> String {
    fn pretty_print(value: &JsonValue, indent: usize, current_indent: usize) -> String {
        let spaces = " ".repeat(current_indent);
        match value {
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
                    result.push_str(&format!("{}{}", spaces, " ".repeat(indent)));
                    result.push_str(&pretty_print(item, indent, current_indent + indent));
                    if i < arr.len() - 1 {
                        result.push(',');
                    }
                    result.push('\n');
                }
                result.push_str(&spaces);
                result.push(']');
                result
            }
            JsonValue::Object(obj) => {
                if obj.is_empty() {
                    return "{}".to_string();
                }
                let mut result = "{\n".to_string();
                let mut entries: Vec<_> = obj.iter().collect();
                entries.sort_by(|a, b| a.0.cmp(b.0));
                
                for (i, (key, value)) in entries.iter().enumerate() {
                    result.push_str(&format!("{}{}\"{}\": ", spaces, " ".repeat(indent), key));
                    result.push_str(&pretty_print(value, indent, current_indent + indent));
                    if i < entries.len() - 1 {
                        result.push(',');
                    }
                    result.push('\n');
                }
                result.push_str(&spaces);
                result.push('}');
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
    fn test_parse_null() {
        assert_eq!(parse_json("null").unwrap(), JsonValue::Null);
    }
    
    #[test]
    fn test_parse_bool() {
        assert_eq!(parse_json("true").unwrap(), JsonValue::Bool(true));
        assert_eq!(parse_json("false").unwrap(), JsonValue::Bool(false));
    }
    
    #[test]
    fn test_parse_number() {
        assert_eq!(parse_json("42").unwrap(), JsonValue::Number(42.0));
        assert_eq!(parse_json("-3.14").unwrap(), JsonValue::Number(-3.14));
    }
    
    #[test]
    fn test_parse_string() {
        assert_eq!(
            parse_json("\"hello\"").unwrap(),
            JsonValue::String("hello".to_string())
        );
    }
    
    #[test]
    fn test_parse_array() {
        let result = parse_json("[1, 2, 3]").unwrap();
        if let JsonValue::Array(arr) = result {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], JsonValue::Number(1.0));
        } else {
            panic!("Expected array");
        }
    }
    
    #[test]
    fn test_parse_object() {
        let result = parse_json("{\"key\": \"value\"}").unwrap();
        if let JsonValue::Object(obj) = result {
            assert_eq!(obj.len(), 1);
            assert_eq!(obj.get("key"), Some(&JsonValue::String("value".to_string())));
        } else {
            panic!("Expected object");
        }
    }
    
    #[test]
    fn test_pretty_print() {
        let json = parse_json("{\"name\": \"John\", \"age\": 30}").unwrap();
        let printed = pretty_print_json(&json, 2);
        assert!(printed.contains("\"name\": \"John\""));
        assert!(printed.contains("\"age\": 30"));
    }
}
use std::collections::HashMap;

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
        Self {
            input: input.chars().collect(),
            position: 0,
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        let value = self.parse_value()?;
        self.skip_whitespace();
        
        if self.position < self.input.len() {
            return Err("Unexpected trailing characters".to_string());
        }
        
        Ok(value)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        
        match self.peek_char() {
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
            Err("Expected boolean value".to_string())
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.consume_char('"');
        let mut result = String::new();
        
        while let Some(c) = self.next_char() {
            if c == '"' {
                break;
            }
            if c == '\\' {
                if let Some(escaped) = self.next_char() {
                    match escaped {
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        '/' => result.push('/'),
                        'b' => result.push('\u{0008}'),
                        'f' => result.push('\u{000C}'),
                        'n' => result.push('\n'),
                        'r' => result.push('\r'),
                        't' => result.push('\t'),
                        _ => return Err("Invalid escape sequence".to_string()),
                    }
                } else {
                    return Err("Unterminated string".to_string());
                }
            } else {
                result.push(c);
            }
        }
        
        Ok(JsonValue::String(result))
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.position;
        let mut has_dot = false;
        let mut has_exponent = false;
        
        if self.peek_char() == Some('-') {
            self.next_char();
        }
        
        while let Some(c) = self.peek_char() {
            if c.is_digit(10) {
                self.next_char();
            } else if c == '.' && !has_dot && !has_exponent {
                has_dot = true;
                self.next_char();
            } else if (c == 'e' || c == 'E') && !has_exponent {
                has_exponent = true;
                self.next_char();
                
                if self.peek_char() == Some('+') || self.peek_char() == Some('-') {
                    self.next_char();
                }
            } else {
                break;
            }
        }
        
        let number_str: String = self.input[start..self.position].iter().collect();
        match number_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err("Invalid number format".to_string()),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.consume_char('[');
        self.skip_whitespace();
        
        let mut array = Vec::new();
        
        if self.peek_char() != Some(']') {
            loop {
                let value = self.parse_value()?;
                array.push(value);
                
                self.skip_whitespace();
                if self.peek_char() == Some(']') {
                    break;
                }
                
                if self.consume_char(',') {
                    self.skip_whitespace();
                } else {
                    return Err("Expected ',' or ']' in array".to_string());
                }
            }
        }
        
        self.consume_char(']');
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.consume_char('{');
        self.skip_whitespace();
        
        let mut object = HashMap::new();
        
        if self.peek_char() != Some('}') {
            loop {
                let key = match self.parse_value()? {
                    JsonValue::String(s) => s,
                    _ => return Err("Object keys must be strings".to_string()),
                };
                
                self.skip_whitespace();
                if !self.consume_char(':') {
                    return Err("Expected ':' after object key".to_string());
                }
                
                self.skip_whitespace();
                let value = self.parse_value()?;
                object.insert(key, value);
                
                self.skip_whitespace();
                if self.peek_char() == Some('}') {
                    break;
                }
                
                if self.consume_char(',') {
                    self.skip_whitespace();
                } else {
                    return Err("Expected ',' or '}' in object".to_string());
                }
            }
        }
        
        self.consume_char('}');
        Ok(JsonValue::Object(object))
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn next_char(&mut self) -> Option<char> {
        if self.position < self.input.len() {
            let c = self.input[self.position];
            self.position += 1;
            Some(c)
        } else {
            None
        }
    }

    fn consume_char(&mut self, expected: char) -> bool {
        if self.peek_char() == Some(expected) {
            self.next_char();
            true
        } else {
            false
        }
    }

    fn consume_str(&mut self, expected: &str) -> bool {
        let start = self.position;
        for expected_char in expected.chars() {
            if self.peek_char() != Some(expected_char) {
                self.position = start;
                return false;
            }
            self.next_char();
        }
        true
    }
}

pub fn pretty_print_json(value: &JsonValue, indent: usize) -> String {
    match value {
        JsonValue::Null => "null".to_string(),
        JsonValue::Bool(b) => b.to_string(),
        JsonValue::Number(n) => n.to_string(),
        JsonValue::String(s) => format!("\"{}\"", s),
        JsonValue::Array(arr) => {
            if arr.is_empty() {
                "[]".to_string()
            } else {
                let items: Vec<String> = arr.iter()
                    .map(|item| format!("{}{}", " ".repeat(indent + 2), pretty_print_json(item, indent + 2)))
                    .collect();
                format!("[\n{}\n{}]", items.join(",\n"), " ".repeat(indent))
            }
        },
        JsonValue::Object(obj) => {
            if obj.is_empty() {
                "{}".to_string()
            } else {
                let items: Vec<String> = obj.iter()
                    .map(|(key, value)| {
                        format!("{}{}: {}", " ".repeat(indent + 2), pretty_print_json(&JsonValue::String(key.clone()), 0), pretty_print_json(value, indent + 2))
                    })
                    .collect();
                format!("{{\n{}\n{}}}", items.join(",\n"), " ".repeat(indent))
            }
        },
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
        let mut parser = JsonParser::new("\"hello world\"");
        assert_eq!(parser.parse(), Ok(JsonValue::String("hello world".to_string())));
    }

    #[test]
    fn test_parse_number() {
        let mut parser = JsonParser::new("42");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(42.0)));
        
        let mut parser = JsonParser::new("-3.14");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(-3.14)));
    }

    #[test]
    fn test_parse_array() {
        let mut parser = JsonParser::new("[1, 2, 3]");
        let expected = JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::Number(2.0),
            JsonValue::Number(3.0),
        ]);
        assert_eq!(parser.parse(), Ok(expected));
    }

    #[test]
    fn test_parse_object() {
        let mut parser = JsonParser::new(r#"{"name": "John", "age": 30}"#);
        let mut expected = HashMap::new();
        expected.insert("name".to_string(), JsonValue::String("John".to_string()));
        expected.insert("age".to_string(), JsonValue::Number(30.0));
        assert_eq!(parser.parse(), Ok(JsonValue::Object(expected)));
    }

    #[test]
    fn test_pretty_print() {
        let json = JsonValue::Object({
            let mut map = HashMap::new();
            map.insert("name".to_string(), JsonValue::String("Alice".to_string()));
            map.insert("scores".to_string(), JsonValue::Array(vec![
                JsonValue::Number(95.0),
                JsonValue::Number(87.0),
                JsonValue::Number(92.0),
            ]));
            map
        });
        
        let printed = pretty_print_json(&json, 0);
        assert!(printed.contains("Alice"));
        assert!(printed.contains("95"));
        assert!(printed.contains("87"));
        assert!(printed.contains("92"));
    }
}