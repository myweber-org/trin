use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

#[derive(Debug)]
pub enum ParseError {
    IoError(String),
    ParseError(String),
    ValidationError(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::IoError(msg) => write!(f, "IO error: {}", msg),
            ParseError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ParseError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ParseError {}

pub struct CsvParser {
    delimiter: char,
    has_header: bool,
}

impl CsvParser {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvParser {
            delimiter,
            has_header,
        }
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<CsvRecord>, ParseError> {
        let file = File::open(&path).map_err(|e| {
            ParseError::IoError(format!("Failed to open file: {}", e))
        })?;

        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line = line.map_err(|e| {
                ParseError::IoError(format!("Failed to read line {}: {}", line_number, e))
            })?;

            if line_number == 1 && self.has_header {
                continue;
            }

            if line.trim().is_empty() {
                continue;
            }

            let record = self.parse_line(&line, line_number)?;
            records.push(record);
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_number: usize) -> Result<CsvRecord, ParseError> {
        let parts: Vec<&str> = line.split(self.delimiter).collect();
        
        if parts.len() != 4 {
            return Err(ParseError::ParseError(format!(
                "Line {}: Expected 4 fields, found {}", 
                line_number, parts.len()
            )));
        }

        let id = parts[0].parse::<u32>().map_err(|_| {
            ParseError::ParseError(format!("Line {}: Invalid ID format", line_number))
        })?;

        let name = parts[1].trim().to_string();
        if name.is_empty() {
            return Err(ParseError::ValidationError(format!(
                "Line {}: Name cannot be empty", line_number
            )));
        }

        let value = parts[2].parse::<f64>().map_err(|_| {
            ParseError::ParseError(format!("Line {}: Invalid value format", line_number))
        })?;

        let active = match parts[3].trim().to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(ParseError::ParseError(format!(
                "Line {}: Invalid boolean value", line_number
            ))),
        };

        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }

    pub fn validate_records(&self, records: &[CsvRecord]) -> Result<(), ParseError> {
        let mut seen_ids = std::collections::HashSet::new();
        
        for record in records {
            if !seen_ids.insert(record.id) {
                return Err(ParseError::ValidationError(format!(
                    "Duplicate ID found: {}", record.id
                )));
            }

            if record.value < 0.0 {
                return Err(ParseError::ValidationError(format!(
                    "Record ID {} has negative value: {}", record.id, record.value
                )));
            }
        }

        Ok(())
    }
}

pub fn calculate_statistics(records: &[CsvRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;

    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;

    let std_dev = variance.sqrt();

    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_parsing() {
        let csv_content = "id,name,value,active\n1,Test1,10.5,true\n2,Test2,20.0,false\n";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_content).unwrap();
        
        let parser = CsvParser::new(',', true);
        let result = parser.parse_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "Test1");
        assert_eq!(records[1].value, 20.0);
    }

    #[test]
    fn test_validation() {
        let records = vec![
            CsvRecord { id: 1, name: "A".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "B".to_string(), value: 20.0, active: false },
        ];
        
        let parser = CsvParser::new(',', false);
        let result = parser.validate_records(&records);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_statistics() {
        let records = vec![
            CsvRecord { id: 1, name: "A".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "B".to_string(), value: 20.0, active: false },
            CsvRecord { id: 3, name: "C".to_string(), value: 30.0, active: true },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}