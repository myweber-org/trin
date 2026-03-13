use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
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

impl From<std::io::Error> for ParseError {
    fn from(error: std::io::Error) -> Self {
        ParseError::IoError(error.to_string())
    }
}

pub fn parse_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<CsvRecord>, ParseError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line_content = line?;
        
        if line_content.trim().is_empty() || line_content.starts_with('#') {
            continue;
        }

        let record = parse_csv_line(&line_content, line_number)?;
        records.push(record);
    }

    if records.is_empty() {
        return Err(ParseError::ValidationError("No valid records found".to_string()));
    }

    Ok(records)
}

fn parse_csv_line(line: &str, line_number: usize) -> Result<CsvRecord, ParseError> {
    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
    
    if parts.len() != 4 {
        return Err(ParseError::ParseError(
            format!("Line {}: Expected 4 columns, found {}", line_number, parts.len())
        ));
    }

    let id = parts[0].parse::<u32>()
        .map_err(|e| ParseError::ParseError(
            format!("Line {}: Invalid ID format: {}", line_number, e)
        ))?;

    let name = parts[1].to_string();
    if name.is_empty() {
        return Err(ParseError::ValidationError(
            format!("Line {}: Name cannot be empty", line_number)
        ));
    }

    let value = parts[2].parse::<f64>()
        .map_err(|e| ParseError::ParseError(
            format!("Line {}: Invalid value format: {}", line_number, e)
        ))?;

    if value < 0.0 {
        return Err(ParseError::ValidationError(
            format!("Line {}: Value cannot be negative", line_number)
        ));
    }

    let active = parts[3].parse::<bool>()
        .map_err(|e| ParseError::ParseError(
            format!("Line {}: Invalid boolean format: {}", line_number, e)
        ))?;

    Ok(CsvRecord {
        id,
        name,
        value,
        active,
    })
}

pub fn calculate_statistics(records: &[CsvRecord]) -> (f64, f64, usize) {
    if records.is_empty() {
        return (0.0, 0.0, 0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len();
    let average = sum / count as f64;
    
    let active_count = records.iter().filter(|r| r.active).count();

    (sum, average, active_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,42.5,true").unwrap();
        writeln!(temp_file, "2,Bob,33.7,false").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "3,Charlie,15.2,true").unwrap();

        let records = parse_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "Alice");
        assert_eq!(records[1].value, 33.7);
        assert!(records[2].active);
    }

    #[test]
    fn test_invalid_csv_format() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,42.5").unwrap(); // Missing column

        let result = parse_csv_file(temp_file.path());
        assert!(matches!(result, Err(ParseError::ParseError(_))));
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            CsvRecord { id: 1, name: "A".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "B".to_string(), value: 20.0, active: false },
            CsvRecord { id: 3, name: "C".to_string(), value: 30.0, active: true },
        ];

        let (sum, average, active_count) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(average, 20.0);
        assert_eq!(active_count, 2);
    }
}