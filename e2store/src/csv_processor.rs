
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
    MissingColumn(String),
}

impl fmt::Display for CsvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CsvError::IoError(e) => write!(f, "IO error: {}", e),
            CsvError::ParseError(msg, line) => write!(f, "Parse error at line {}: {}", line, msg),
            CsvError::InvalidHeader(msg) => write!(f, "Invalid header: {}", msg),
            CsvError::MissingColumn(col) => write!(f, "Missing required column: {}", col),
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
    required_columns: Vec<String>,
    delimiter: char,
}

impl CsvProcessor {
    pub fn new(required_columns: Vec<&str>, delimiter: char) -> Self {
        CsvProcessor {
            required_columns: required_columns.into_iter().map(String::from).collect(),
            delimiter,
        }
    }

    pub fn validate_file<P: AsRef<Path>>(&self, file_path: P) -> Result<(), CsvError> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        let mut lines = reader.lines().enumerate();
        
        if let Some((line_num, line)) = lines.next() {
            let header = line?;
            self.validate_header(&header, line_num + 1)?;
        }
        
        for (line_num, line) in lines {
            let line_content = line?;
            self.validate_line(&line_content, line_num + 1)?;
        }
        
        Ok(())
    }
    
    fn validate_header(&self, header: &str, line_num: usize) -> Result<(), CsvError> {
        let columns: Vec<&str> = header.split(self.delimiter).collect();
        
        for required in &self.required_columns {
            if !columns.contains(&required.as_str()) {
                return Err(CsvError::MissingColumn(required.clone()));
            }
        }
        
        if columns.len() != columns.iter().collect::<std::collections::HashSet<_>>().len() {
            return Err(CsvError::InvalidHeader(
                format!("Duplicate column names found at line {}", line_num)
            ));
        }
        
        Ok(())
    }
    
    fn validate_line(&self, line: &str, line_num: usize) -> Result<(), CsvError> {
        let fields: Vec<&str> = line.split(self.delimiter).collect();
        
        if fields.len() != self.required_columns.len() {
            return Err(CsvError::ParseError(
                format!("Expected {} fields, found {}", self.required_columns.len(), fields.len()),
                line_num
            ));
        }
        
        for (i, field) in fields.iter().enumerate() {
            if field.trim().is_empty() {
                return Err(CsvError::ParseError(
                    format!("Empty field at column {}", i + 1),
                    line_num
                ));
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,email").unwrap();
        writeln!(temp_file, "John Doe,30,john@example.com").unwrap();
        
        let processor = CsvProcessor::new(vec!["name", "age", "email"], ',');
        let result = processor.validate_file(temp_file.path());
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_missing_column() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age").unwrap();
        
        let processor = CsvProcessor::new(vec!["name", "age", "email"], ',');
        let result = processor.validate_file(temp_file.path());
        assert!(matches!(result, Err(CsvError::MissingColumn(_))));
    }
    
    #[test]
    fn test_empty_field() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,email").unwrap();
        writeln!(temp_file, "John Doe,,john@example.com").unwrap();
        
        let processor = CsvProcessor::new(vec!["name", "age", "email"], ',');
        let result = processor.validate_file(temp_file.path());
        assert!(matches!(result, Err(CsvError::ParseError(_, _))));
    }
}