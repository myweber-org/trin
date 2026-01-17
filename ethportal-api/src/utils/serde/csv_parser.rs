use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub enum CsvError {
    IoError(std::io::Error),
    ParseError(String, usize),
    InconsistentColumns(usize, usize, usize),
}

impl fmt::Display for CsvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CsvError::IoError(e) => write!(f, "IO error: {}", e),
            CsvError::ParseError(msg, line) => write!(f, "Parse error at line {}: {}", line, msg),
            CsvError::InconsistentColumns(expected, actual, line) => write!(
                f,
                "Inconsistent columns at line {}: expected {}, found {}",
                line, expected, actual
            ),
        }
    }
}

impl Error for CsvError {}

impl From<std::io::Error> for CsvError {
    fn from(error: std::io::Error) -> Self {
        CsvError::IoError(error)
    }
}

pub struct CsvParser {
    delimiter: char,
    has_header: bool,
}

impl Default for CsvParser {
    fn default() -> Self {
        CsvParser {
            delimiter: ',',
            has_header: true,
        }
    }
}

impl CsvParser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn has_header(mut self, has_header: bool) -> Self {
        self.has_header = has_header;
        self
    }

    pub fn parse_file(&self, path: &str) -> Result<Vec<Vec<String>>, CsvError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        self.parse(reader)
    }

    pub fn parse<R: BufRead>(&self, reader: R) -> Result<Vec<Vec<String>>, CsvError> {
        let mut records = Vec::new();
        let mut lines = reader.lines().enumerate();
        let mut expected_columns: Option<usize> = None;

        if self.has_header {
            if let Some((line_num, line)) = lines.next() {
                let line = line.map_err(|e| CsvError::IoError(e))?;
                let headers: Vec<String> = self.parse_line(&line, line_num + 1)?;
                expected_columns = Some(headers.len());
                records.push(headers);
            }
        }

        for (line_num, line) in lines {
            let line_num = line_num + 1;
            let line = line.map_err(|e| CsvError::IoError(e))?;
            let fields: Vec<String> = self.parse_line(&line, line_num)?;

            if let Some(expected) = expected_columns {
                if fields.len() != expected {
                    return Err(CsvError::InconsistentColumns(expected, fields.len(), line_num));
                }
            } else {
                expected_columns = Some(fields.len());
            }

            records.push(fields);
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<Vec<String>, CsvError> {
        let mut fields = Vec::new();
        let mut current_field = String::new();
        let mut chars = line.chars().peekable();
        let mut in_quotes = false;

        while let Some(ch) = chars.next() {
            match ch {
                '"' => {
                    if in_quotes && chars.peek() == Some(&'"') {
                        current_field.push('"');
                        chars.next();
                    } else {
                        in_quotes = !in_quotes;
                    }
                }
                _ if ch == self.delimiter && !in_quotes => {
                    fields.push(current_field.trim().to_string());
                    current_field.clear();
                }
                _ => {
                    current_field.push(ch);
                }
            }
        }

        fields.push(current_field.trim().to_string());

        if in_quotes {
            return Err(CsvError::ParseError(
                "Unclosed quotation mark".to_string(),
                line_num,
            ));
        }

        Ok(fields)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let data = "name,age,city\nJohn,30,New York\nJane,25,London";
        let parser = CsvParser::new();
        let result = parser.parse(data.as_bytes()).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], vec!["name", "age", "city"]);
        assert_eq!(result[1], vec!["John", "30", "New York"]);
        assert_eq!(result[2], vec!["Jane", "25", "London"]);
    }

    #[test]
    fn test_quoted_fields() {
        let data = "\"John, Doe\",30,\"New, York\"";
        let parser = CsvParser::new().has_header(false);
        let result = parser.parse(data.as_bytes()).unwrap();

        assert_eq!(result[0], vec!["John, Doe", "30", "New, York"]);
    }

    #[test]
    fn test_inconsistent_columns() {
        let data = "name,age,city\nJohn,30\nJane,25,London,extra";
        let parser = CsvParser::new();
        let result = parser.parse(data.as_bytes());

        assert!(matches!(result, Err(CsvError::InconsistentColumns(_, _, _))));
    }
}