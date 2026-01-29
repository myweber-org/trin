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
    IoError(String),
    FormatError(String),
    ValidationError(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::IoError(msg) => write!(f, "IO error: {}", msg),
            ParseError::FormatError(msg) => write!(f, "Format error: {}", msg),
            ParseError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ParseError {}

pub fn parse_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, ParseError> {
    let file = File::open(path.as_ref())
        .map_err(|e| ParseError::IoError(format!("Failed to open file: {}", e)))?;
    
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    
    for (line_num, line) in reader.lines().enumerate() {
        let line = line
            .map_err(|e| ParseError::IoError(format!("Failed to read line {}: {}", line_num + 1, e)))?;
        
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        
        let record = parse_csv_line(&line, line_num + 1)?;
        records.push(record);
    }
    
    Ok(records)
}

fn parse_csv_line(line: &str, line_num: usize) -> Result<Record, ParseError> {
    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
    
    if parts.len() != 4 {
        return Err(ParseError::FormatError(
            format!("Line {}: Expected 4 fields, found {}", line_num, parts.len())
        ));
    }
    
    let id = parts[0].parse::<u32>()
        .map_err(|_| ParseError::ValidationError(
            format!("Line {}: Invalid ID format '{}'", line_num, parts[0])
        ))?;
    
    let name = parts[1].to_string();
    if name.is_empty() {
        return Err(ParseError::ValidationError(
            format!("Line {}: Name cannot be empty", line_num)
        ));
    }
    
    let value = parts[2].parse::<f64>()
        .map_err(|_| ParseError::ValidationError(
            format!("Line {}: Invalid value format '{}'", line_num, parts[2])
        ))?;
    
    if value < 0.0 {
        return Err(ParseError::ValidationError(
            format!("Line {}: Value cannot be negative: {}", line_num, value)
        ));
    }
    
    let active = match parts[3].to_lowercase().as_str() {
        "true" | "1" | "yes" => true,
        "false" | "0" | "no" => false,
        _ => return Err(ParseError::ValidationError(
            format!("Line {}: Invalid boolean value '{}'", line_num, parts[3])
        )),
    };
    
    Ok(Record { id, name, value, active })
}

pub fn calculate_stats(records: &[Record]) -> (f64, f64, usize) {
    if records.is_empty() {
        return (0.0, 0.0, 0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let avg = sum / records.len() as f64;
    let active_count = records.iter().filter(|r| r.active).count();
    
    (sum, avg, active_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_parse_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,42.5,true").unwrap();
        writeln!(temp_file, "2,Bob,33.7,false").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "3,Charlie,15.2,yes").unwrap();
        
        let records = parse_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], Record { id: 1, name: "Alice".to_string(), value: 42.5, active: true });
        assert_eq!(records[1], Record { id: 2, name: "Bob".to_string(), value: 33.7, active: false });
        assert_eq!(records[2], Record { id: 3, name: "Charlie".to_string(), value: 15.2, active: true });
    }
    
    #[test]
    fn test_parse_invalid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "invalid,Alice,42.5,true").unwrap();
        
        let result = parse_csv_file(temp_file.path());
        assert!(result.is_err());
    }
    
    #[test]
    fn test_calculate_stats() {
        let records = vec![
            Record { id: 1, name: "Alice".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "Bob".to_string(), value: 20.0, active: false },
            Record { id: 3, name: "Charlie".to_string(), value: 30.0, active: true },
        ];
        
        let (sum, avg, active_count) = calculate_stats(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(avg, 20.0);
        assert_eq!(active_count, 2);
    }
}