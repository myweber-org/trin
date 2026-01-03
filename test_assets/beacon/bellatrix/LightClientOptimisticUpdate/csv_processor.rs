use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub enum CsvError {
    IoError(std::io::Error),
    ParseError(String, usize),
    InvalidHeader(String),
    EmptyFile,
}

impl fmt::Display for CsvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CsvError::IoError(e) => write!(f, "IO error: {}", e),
            CsvError::ParseError(msg, line) => write!(f, "Parse error at line {}: {}", line, msg),
            CsvError::InvalidHeader(msg) => write!(f, "Invalid header: {}", msg),
            CsvError::EmptyFile => write!(f, "File is empty"),
        }
    }
}

impl Error for CsvError {}

impl From<std::io::Error> for CsvError {
    fn from(error: std::io::Error) -> Self {
        CsvError::IoError(error)
    }
}

pub struct CsvProcessor {
    delimiter: char,
    has_header: bool,
}

impl Default for CsvProcessor {
    fn default() -> Self {
        CsvProcessor {
            delimiter: ',',
            has_header: true,
        }
    }
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, CsvError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        let mut records = Vec::new();
        let mut line_number = 0;

        if let Some(first_line) = lines.next() {
            let first_line = first_line?;
            line_number += 1;
            
            if first_line.trim().is_empty() {
                return Err(CsvError::EmptyFile);
            }

            let first_record = self.parse_line(&first_line, line_number)?;
            
            if self.has_header {
                self.validate_header(&first_record)?;
            } else {
                records.push(first_record);
            }
        } else {
            return Err(CsvError::EmptyFile);
        }

        for line in lines {
            let line = line?;
            line_number += 1;
            
            if line.trim().is_empty() {
                continue;
            }
            
            let record = self.parse_line(&line, line_number)?;
            records.push(record);
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_number: usize) -> Result<Vec<String>, CsvError> {
        let mut fields = Vec::new();
        let mut current_field = String::new();
        let mut in_quotes = false;
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];
            
            if c == '"' {
                if in_quotes && i + 1 < chars.len() && chars[i + 1] == '"' {
                    current_field.push('"');
                    i += 1;
                } else {
                    in_quotes = !in_quotes;
                }
            } else if c == self.delimiter && !in_quotes {
                fields.push(current_field.trim().to_string());
                current_field.clear();
            } else {
                current_field.push(c);
            }
            
            i += 1;
        }

        fields.push(current_field.trim().to_string());

        if in_quotes {
            return Err(CsvError::ParseError(
                "Unclosed quotation mark".to_string(),
                line_number,
            ));
        }

        Ok(fields)
    }

    fn validate_header(&self, header: &[String]) -> Result<(), CsvError> {
        if header.is_empty() {
            return Err(CsvError::InvalidHeader("Header cannot be empty".to_string()));
        }

        let mut seen_fields = std::collections::HashSet::new();
        for field in header {
            let trimmed = field.trim();
            if trimmed.is_empty() {
                return Err(CsvError::InvalidHeader(
                    "Header fields cannot be empty".to_string(),
                ));
            }
            if !seen_fields.insert(trimmed) {
                return Err(CsvError::InvalidHeader(
                    format!("Duplicate header field: {}", trimmed),
                ));
            }
        }

        Ok(())
    }

    pub fn count_records(&self, records: &[Vec<String>]) -> usize {
        records.len()
    }

    pub fn get_column_data(&self, records: &[Vec<String>], column_index: usize) -> Option<Vec<String>> {
        if records.is_empty() {
            return None;
        }

        let mut column_data = Vec::new();
        for record in records {
            if column_index < record.len() {
                column_data.push(record[column_index].clone());
            }
        }

        Some(column_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "\"Charlie, Jr.\",35,\"Paris, France\"").unwrap();

        let processor = CsvProcessor::default();
        let result = processor.process_file(temp_file.path());
        assert!(result.is_ok());
        
        let records = result.unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["Alice", "30", "New York"]);
        assert_eq!(records[2], vec!["Charlie, Jr.", "35", "Paris, France"]);
    }

    #[test]
    fn test_invalid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,\"New York").unwrap();

        let processor = CsvProcessor::default();
        let result = processor.process_file(temp_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let processor = CsvProcessor::default();
        let result = processor.process_file(temp_file.path());
        assert!(matches!(result, Err(CsvError::EmptyFile)));
    }
}