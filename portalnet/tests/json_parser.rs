use std::collections::HashMap;
use std::error::Error;

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
    input: String,
    position: usize,
}

impl JsonParser {
    pub fn new(input: &str) -> Self {
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

    fn parse_string(&mut self) -> Result<String, Box<dyn Error>> {
        self.position += 1; // Skip opening quote
        let start = self.position;
        let mut result = String::new();

        while self.position < self.input.len() {
            let c = self.input.chars().nth(self.position).unwrap();
            if c == '"' {
                let slice = &self.input[start..self.position];
                result.push_str(slice);
                self.position += 1;
                return Ok(result);
            } else if c == '\\' {
                let slice = &self.input[start..self.position];
                result.push_str(slice);
                self.position += 1;
                if self.position < self.input.len() {
                    let escaped = self.input.chars().nth(self.position).unwrap();
                    result.push(match escaped {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '"' => '"',
                        _ => escaped,
                    });
                    self.position += 1;
                }
            } else {
                self.position += 1;
            }
        }

        Err("Unterminated string".into())
    }

    fn parse_number(&mut self) -> Result<f64, Box<dyn Error>> {
        let start = self.position;
        while self.position < self.input.len() {
            let c = self.input.chars().nth(self.position).unwrap();
            if c.is_digit(10) || c == '.' || c == '-' || c == '+' || c == 'e' || c == 'E' {
                self.position += 1;
            } else {
                break;
            }
        }
        let slice = &self.input[start..self.position];
        slice.parse::<f64>().map_err(|e| e.into())
    }

    fn parse_array(&mut self) -> Result<Vec<JsonValue>, Box<dyn Error>> {
        self.position += 1; // Skip '['
        self.skip_whitespace();
        let mut array = Vec::new();

        if self.peek() == ']' {
            self.position += 1;
            return Ok(array);
        }

        loop {
            self.skip_whitespace();
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();

            if self.peek() == ']' {
                self.position += 1;
                break;
            } else if self.peek() == ',' {
                self.position += 1;
                continue;
            } else {
                return Err("Expected ',' or ']' in array".into());
            }
        }

        Ok(array)
    }

    fn parse_object(&mut self) -> Result<HashMap<String, JsonValue>, Box<dyn Error>> {
        self.position += 1; // Skip '{'
        self.skip_whitespace();
        let mut map = HashMap::new();

        if self.peek() == '}' {
            self.position += 1;
            return Ok(map);
        }

        loop {
            self.skip_whitespace();
            if self.peek() != '"' {
                return Err("Expected string key in object".into());
            }
            let key = self.parse_string()?;
            self.skip_whitespace();

            if self.peek() != ':' {
                return Err("Expected ':' after key in object".into());
            }
            self.position += 1;
            self.skip_whitespace();

            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();

            if self.peek() == '}' {
                self.position += 1;
                break;
            } else if self.peek() == ',' {
                self.position += 1;
                continue;
            } else {
                return Err("Expected ',' or '}' in object".into());
            }
        }

        Ok(map)
    }

    fn parse_value(&mut self) -> Result<JsonValue, Box<dyn Error>> {
        self.skip_whitespace();
        let c = self.peek();

        match c {
            'n' => {
                if self.input[self.position..].starts_with("null") {
                    self.position += 4;
                    Ok(JsonValue::Null)
                } else {
                    Err("Expected null".into())
                }
            }
            't' => {
                if self.input[self.position..].starts_with("true") {
                    self.position += 4;
                    Ok(JsonValue::Bool(true))
                } else {
                    Err("Expected true".into())
                }
            }
            'f' => {
                if self.input[self.position..].starts_with("false") {
                    self.position += 5;
                    Ok(JsonValue::Bool(false))
                } else {
                    Err("Expected false".into())
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
            c if c.is_digit(10) || c == '-' => {
                let num = self.parse_number()?;
                Ok(JsonValue::Number(num))
            }
            _ => Err(format!("Unexpected character: {}", c).into()),
        }
    }

    fn peek(&self) -> char {
        if self.position < self.input.len() {
            self.input.chars().nth(self.position).unwrap()
        } else {
            '\0'
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, Box<dyn Error>> {
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.position < self.input.len() {
            return Err("Trailing characters after JSON value".into());
        }
        Ok(result)
    }
}

pub fn extract_string_values(json_str: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut parser = JsonParser::new(json_str);
    let json_value = parser.parse()?;
    let mut result = Vec::new();
    collect_strings(&json_value, &mut result);
    Ok(result)
}

fn collect_strings(value: &JsonValue, result: &mut Vec<String>) {
    match value {
        JsonValue::String(s) => result.push(s.clone()),
        JsonValue::Array(arr) => {
            for item in arr {
                collect_strings(item, result);
            }
        }
        JsonValue::Object(obj) => {
            for (_, val) in obj {
                collect_strings(val, result);
            }
        }
        _ => {}
    }
}