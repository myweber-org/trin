use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvParser {
    delimiter: char,
    has_headers: bool,
}

impl CsvParser {
    pub fn new() -> Self {
        CsvParser {
            delimiter: ',',
            has_headers: true,
        }
    }

    pub fn delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn has_headers(mut self, has_headers: bool) -> Self {
        self.has_headers = has_headers;
        self
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.trim().is_empty() {
                continue;
            }

            if line_num == 0 && self.has_headers {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            records.push(fields);
        }

        Ok(records)
    }

    pub fn parse_string(&self, content: &str) -> Vec<Vec<String>> {
        content
            .lines()
            .enumerate()
            .filter(|(line_num, line)| {
                !line.trim().is_empty() && !(self.has_headers && *line_num == 0)
            })
            .map(|(_, line)| {
                line.split(self.delimiter)
                    .map(|s| s.trim().to_string())
                    .collect()
            })
            .collect()
    }
}

impl Default for CsvParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string_with_headers() {
        let parser = CsvParser::new();
        let csv_data = "name,age,city\nJohn,30,New York\nJane,25,London";
        let result = parser.parse_string(csv_data);
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
        assert_eq!(result[1], vec!["Jane", "25", "London"]);
    }

    #[test]
    fn test_parse_string_without_headers() {
        let parser = CsvParser::new().has_headers(false);
        let csv_data = "John,30,New York\nJane,25,London";
        let result = parser.parse_string(csv_data);
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
        assert_eq!(result[1], vec!["Jane", "25", "London"]);
    }

    #[test]
    fn test_custom_delimiter() {
        let parser = CsvParser::new().delimiter(';');
        let csv_data = "name;age;city\nJohn;30;New York";
        let result = parser.parse_string(csv_data);
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_empty_lines_skipped() {
        let parser = CsvParser::new();
        let csv_data = "name,age,city\n\nJohn,30,New York\n\n\nJane,25,London\n";
        let result = parser.parse_string(csv_data);
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
        assert_eq!(result[1], vec!["Jane", "25", "London"]);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

#[derive(Debug)]
pub enum ParseError {
    IoError(std::io::Error),
    InvalidFormat(String),
    InvalidNumber(String),
    InvalidBoolean(String),
    MissingColumn,
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err)
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
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvParser {
            delimiter,
            has_header,
        }
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Record>, ParseError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 && self.has_header {
                continue;
            }

            if line.trim().is_empty() {
                continue;
            }

            let record = self.parse_line(&line, line_num + 1)?;
            records.push(record);
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<Record, ParseError> {
        let parts: Vec<&str> = line.split(self.delimiter).collect();
        
        if parts.len() < 4 {
            return Err(ParseError::MissingColumn);
        }

        let id = parts[0].trim().parse::<u32>()
            .map_err(|_| ParseError::InvalidNumber(format!("Line {}: Invalid ID '{}'", line_num, parts[0])))?;

        let name = parts[1].trim().to_string();
        if name.is_empty() {
            return Err(ParseError::InvalidFormat(format!("Line {}: Name cannot be empty", line_num)));
        }

        let value = parts[2].trim().parse::<f64>()
            .map_err(|_| ParseError::InvalidNumber(format!("Line {}: Invalid value '{}'", line_num, parts[2])))?;

        let active = match parts[3].trim().to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            other => return Err(ParseError::InvalidBoolean(format!("Line {}: Invalid boolean '{}'", line_num, other))),
        };

        Ok(Record {
            id,
            name,
            value,
            active,
        })
    }

    pub fn calculate_stats(records: &[Record]) -> (f64, f64, usize) {
        if records.is_empty() {
            return (0.0, 0.0, 0);
        }

        let sum: f64 = records.iter().map(|r| r.value).sum();
        let mean = sum / records.len() as f64;
        
        let active_count = records.iter().filter(|r| r.active).count();

        (sum, mean, active_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,Test1,10.5,true").unwrap();
        writeln!(temp_file, "2,Test2,20.0,false").unwrap();

        let parser = CsvParser::default();
        let result = parser.parse_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].id, 1);
        assert_eq!(records[0].name, "Test1");
        assert_eq!(records[0].value, 10.5);
        assert_eq!(records[0].active, true);
    }

    #[test]
    fn test_parse_invalid_number() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "abc,Test,10.5,true").unwrap();

        let parser = CsvParser::default();
        let result = parser.parse_file(temp_file.path());
        
        assert!(matches!(result, Err(ParseError::InvalidNumber(_))));
    }

    #[test]
    fn test_calculate_stats() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "B".to_string(), value: 20.0, active: false },
            Record { id: 3, name: "C".to_string(), value: 30.0, active: true },
        ];

        let (sum, mean, active_count) = CsvParser::calculate_stats(&records);
        
        assert_eq!(sum, 60.0);
        assert_eq!(mean, 20.0);
        assert_eq!(active_count, 2);
    }
}