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
            let ch = chars[i];
            
            if ch == '"' {
                if in_quotes && i + 1 < chars.len() && chars[i + 1] == '"' {
                    current_field.push('"');
                    i += 1;
                } else {
                    in_quotes = !in_quotes;
                }
            } else if ch == self.delimiter && !in_quotes {
                fields.push(current_field.trim().to_string());
                current_field.clear();
            } else {
                current_field.push(ch);
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

        let mut seen_columns = std::collections::HashSet::new();
        for (idx, column) in header.iter().enumerate() {
            if column.trim().is_empty() {
                return Err(CsvError::InvalidHeader(
                    format!("Column {} has empty name", idx + 1)
                ));
            }
            
            if seen_columns.contains(column) {
                return Err(CsvError::InvalidHeader(
                    format!("Duplicate column name: {}", column)
                ));
            }
            
            seen_columns.insert(column.clone());
        }

        Ok(())
    }

    pub fn count_records(&self, records: &[Vec<String>]) -> usize {
        records.len()
    }

    pub fn get_column_count(&self, records: &[Vec<String>]) -> Option<usize> {
        records.first().map(|record| record.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Jane,25,London").unwrap();

        let processor = CsvProcessor::default();
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_csv_with_quotes() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,description").unwrap();
        writeln!(temp_file, "1,\"Item, with comma\"").unwrap();
        writeln!(temp_file, "2,\"Normal item\"").unwrap();

        let processor = CsvProcessor::default();
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records[0], vec!["1", "Item, with comma"]);
    }

    #[test]
    fn test_invalid_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,name,age").unwrap();

        let processor = CsvProcessor::default();
        let result = processor.process_file(temp_file.path());
        
        assert!(matches!(result, Err(CsvError::InvalidHeader(_))));
    }
}