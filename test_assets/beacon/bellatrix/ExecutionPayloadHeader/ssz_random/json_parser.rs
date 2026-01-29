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

    fn peek(&self) -> Option<char> {
        if self.position < self.input.len() {
            Some(self.input[self.position])
        } else {
            None
        }
    }

    fn consume(&mut self, expected: char) -> Result<(), String> {
        self.skip_whitespace();
        if let Some(ch) = self.peek() {
            if ch == expected {
                self.position += 1;
                Ok(())
            } else {
                Err(format!("Expected '{}', found '{}'", expected, ch))
            }
        } else {
            Err("Unexpected end of input".to_string())
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.consume('"')?;
        let mut result = String::new();
        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.position += 1;
                return Ok(result);
            } else if ch == '\\' {
                self.position += 1;
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
                    self.position += 1;
                } else {
                    return Err("Incomplete escape sequence".to_string());
                }
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
        num_str.parse::<f64>().map_err(|e| e.to_string())
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
                    return Err(format!("Expected ',' or ']', found '{}'", ch));
                }
            } else {
                return Err("Unexpected end of input in array".to_string());
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
                    return Err(format!("Expected ',' or '}}', found '{}'", ch));
                }
            } else {
                return Err("Unexpected end of input in object".to_string());
            }
        }
        Ok(map)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        if let Some(ch) = self.peek() {
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
                _ => Err(format!("Unexpected character: '{}'", ch)),
            }
        } else {
            Err("Unexpected end of input".to_string())
        }
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

pub fn parse_json(json_str: &str) -> Result<JsonValue, String> {
    let mut parser = JsonParser::new(json_str);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_null() {
        assert_eq!(parse_json("null").unwrap(), JsonValue::Null);
    }

    #[test]
    fn test_parse_boolean() {
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
            assert_eq!(arr[1], JsonValue::Number(2.0));
            assert_eq!(arr[2], JsonValue::Number(3.0));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_parse_object() {
        let result = parse_json(r#"{"key": "value"}"#).unwrap();
        if let JsonValue::Object(map) = result {
            assert_eq!(map.len(), 1);
            assert_eq!(
                map.get("key").unwrap(),
                &JsonValue::String("value".to_string())
            );
        } else {
            panic!("Expected object");
        }
    }
}