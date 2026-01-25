use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
enum CsvError {
    IoError(String),
    ParseError(usize, String),
    InvalidHeader(String),
}

impl fmt::Display for CsvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CsvError::IoError(msg) => write!(f, "IO error: {}", msg),
            CsvError::ParseError(line, msg) => write!(f, "Parse error at line {}: {}", line, msg),
            CsvError::InvalidHeader(msg) => write!(f, "Invalid header: {}", msg),
        }
    }
}

impl Error for CsvError {}

struct CsvProcessor {
    delimiter: char,
    has_header: bool,
}

impl CsvProcessor {
    fn new(delimiter: char, has_header: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_header,
        }
    }

    fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, CsvError> {
        let file = File::open(&path).map_err(|e| CsvError::IoError(e.to_string()))?;
        let reader = BufReader::new(file);
        
        let mut records = Vec::new();
        let mut line_number = 0;
        
        for line in reader.lines() {
            line_number += 1;
            let line_content = line.map_err(|e| CsvError::IoError(e.to_string()))?;
            
            if line_content.trim().is_empty() {
                continue;
            }
            
            let fields: Vec<String> = line_content
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if fields.is_empty() {
                return Err(CsvError::ParseError(line_number, "Empty line".to_string()));
            }
            
            if self.has_header && line_number == 1 {
                if fields.iter().any(|f| f.is_empty()) {
                    return Err(CsvError::InvalidHeader("Empty header field".to_string()));
                }
            }
            
            records.push(fields);
        }
        
        if records.is_empty() {
            return Err(CsvError::ParseError(0, "File contains no data".to_string()));
        }
        
        Ok(records)
    }
    
    fn validate_records(&self, records: &[Vec<String>]) -> Result<(), CsvError> {
        if records.is_empty() {
            return Ok(());
        }
        
        let expected_len = records[0].len();
        
        for (idx, record) in records.iter().enumerate() {
            let actual_line = if self.has_header { idx + 2 } else { idx + 1 };
            
            if record.len() != expected_len {
                return Err(CsvError::ParseError(
                    actual_line,
                    format!("Expected {} fields, found {}", expected_len, record.len())
                ));
            }
            
            if record.iter().any(|field| field.is_empty()) {
                return Err(CsvError::ParseError(
                    actual_line,
                    "Empty field detected".to_string()
                ));
            }
        }
        
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let processor = CsvProcessor::new(',', true);
    
    match processor.process_file("data.csv") {
        Ok(records) => {
            if let Err(e) = processor.validate_records(&records) {
                eprintln!("Validation error: {}", e);
                return Err(Box::new(e));
            }
            
            println!("Successfully processed {} records", records.len());
            
            if !records.is_empty() {
                println!("Headers: {:?}", records[0]);
                if records.len() > 1 {
                    println!("First data row: {:?}", records[1]);
                }
            }
        }
        Err(e) => {
            eprintln!("Processing error: {}", e);
            return Err(Box::new(e));
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        assert!(result.is_ok());
        
        let records = result.unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["name", "age", "city"]);
    }
    
    #[test]
    fn test_invalid_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,,city").unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        assert!(matches!(result, Err(CsvError::InvalidHeader(_))));
    }
    
    #[test]
    fn test_missing_field() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30").unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        assert!(result.is_ok());
        
        let records = result.unwrap();
        let validation_result = processor.validate_records(&records);
        assert!(matches!(validation_result, Err(CsvError::ParseError(_, _))));
    }
}