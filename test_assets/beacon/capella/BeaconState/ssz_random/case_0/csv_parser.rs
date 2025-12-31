use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub fields: Vec<String>,
}

#[derive(Debug)]
pub struct CsvParser {
    pub records: Vec<CsvRecord>,
    pub delimiter: char,
}

impl CsvParser {
    pub fn new() -> Self {
        CsvParser {
            records: Vec::new(),
            delimiter: ',',
        }
    }

    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn parse_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            self.parse_line(&line)?;
        }

        Ok(())
    }

    pub fn parse_line(&mut self, line: &str) -> Result<(), Box<dyn Error>> {
        let mut fields = Vec::new();
        let mut current_field = String::new();
        let mut inside_quotes = false;
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];

            if c == '"' {
                if inside_quotes && i + 1 < chars.len() && chars[i + 1] == '"' {
                    current_field.push('"');
                    i += 1;
                } else {
                    inside_quotes = !inside_quotes;
                }
            } else if c == self.delimiter && !inside_quotes {
                fields.push(current_field.clone());
                current_field.clear();
            } else {
                current_field.push(c);
            }

            i += 1;
        }

        fields.push(current_field);

        if inside_quotes {
            return Err("Unclosed quotes in CSV line".into());
        }

        self.records.push(CsvRecord { fields });
        Ok(())
    }

    pub fn get_records(&self) -> &[CsvRecord] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_parsing() {
        let mut parser = CsvParser::new();
        let result = parser.parse_line("name,age,city");
        assert!(result.is_ok());
        assert_eq!(parser.records.len(), 1);
        assert_eq!(parser.records[0].fields, vec!["name", "age", "city"]);
    }

    #[test]
    fn test_quoted_fields() {
        let mut parser = CsvParser::new();
        let result = parser.parse_line(r#""John Doe",30,"New York, NY""#);
        assert!(result.is_ok());
        assert_eq!(parser.records.len(), 1);
        assert_eq!(
            parser.records[0].fields,
            vec!["John Doe", "30", "New York, NY"]
        );
    }

    #[test]
    fn test_empty_lines() {
        let mut parser = CsvParser::new();
        let result = parser.parse_line("");
        assert!(result.is_err());
    }

    #[test]
    fn test_file_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, r#""Alice",25,"London""#).unwrap();
        writeln!(temp_file, r#""Bob",30,"Paris""#).unwrap();

        let mut parser = CsvParser::new();
        let result = parser.parse_file(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(parser.records.len(), 3);
    }
}