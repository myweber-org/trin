use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

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

    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn without_headers(mut self) -> Self {
        self.has_headers = false;
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

            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if self.has_headers && line_num == 0 {
                continue;
            }

            records.push(record);
        }

        Ok(records)
    }

    pub fn parse_string(&self, data: &str) -> Vec<Vec<String>> {
        let mut records = Vec::new();
        
        for (line_num, line) in data.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }

            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if self.has_headers && line_num == 0 {
                continue;
            }

            records.push(record);
        }

        records
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
    fn test_basic_parsing() {
        let parser = CsvParser::new();
        let data = "name,age,city\nJohn,30,New York\nJane,25,London";
        let result = parser.parse_string(data);
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
        assert_eq!(result[1], vec!["Jane", "25", "London"]);
    }

    #[test]
    fn test_custom_delimiter() {
        let parser = CsvParser::new().with_delimiter(';');
        let data = "name;age;city\nJohn;30;New York";
        let result = parser.parse_string(data);
        
        assert_eq!(result[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_without_headers() {
        let parser = CsvParser::new().without_headers();
        let data = "John,30,New York\nJane,25,London";
        let result = parser.parse_string(data);
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_empty_lines_skipped() {
        let parser = CsvParser::new();
        let data = "name,age\n\nJohn,30\n\nJane,25\n";
        let result = parser.parse_string(data);
        
        assert_eq!(result.len(), 2);
    }
}