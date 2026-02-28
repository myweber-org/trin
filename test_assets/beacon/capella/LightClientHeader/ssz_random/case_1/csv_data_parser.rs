
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
    ParseError(String, usize),
    ValidationError(String, usize),
}

pub fn parse_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<CsvRecord>, ParseError> {
    let file = File::open(path).map_err(|e| ParseError::IoError(e.to_string()))?;
    let reader = BufReader::new(file);
    
    let mut records = Vec::new();
    let mut line_number = 0;
    
    for line in reader.lines() {
        line_number += 1;
        let line_content = line.map_err(|e| ParseError::IoError(e.to_string()))?;
        
        if line_content.trim().is_empty() || line_content.starts_with('#') {
            continue;
        }
        
        let record = parse_csv_line(&line_content, line_number)?;
        records.push(record);
    }
    
    Ok(records)
}

fn parse_csv_line(line: &str, line_number: usize) -> Result<CsvRecord, ParseError> {
    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
    
    if parts.len() != 4 {
        return Err(ParseError::ParseError(
            format!("Expected 4 columns, found {}", parts.len()),
            line_number
        ));
    }
    
    let id = parts[0].parse::<u32>()
        .map_err(|_| ParseError::ParseError(
            format!("Invalid ID format: {}", parts[0]),
            line_number
        ))?;
    
    let name = parts[1].to_string();
    if name.is_empty() {
        return Err(ParseError::ValidationError(
            "Name cannot be empty".to_string(),
            line_number
        ));
    }
    
    let value = parts[2].parse::<f64>()
        .map_err(|_| ParseError::ParseError(
            format!("Invalid value format: {}", parts[2]),
            line_number
        ))?;
    
    let active = match parts[3].to_lowercase().as_str() {
        "true" | "1" | "yes" => true,
        "false" | "0" | "no" => false,
        _ => return Err(ParseError::ParseError(
            format!("Invalid boolean format: {}", parts[3]),
            line_number
        )),
    };
    
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
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - average).powi(2))
        .sum::<f64>() / count as f64;
    
    (average, variance.sqrt(), count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_parse_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,ItemA,42.5,true").unwrap();
        writeln!(temp_file, "2,ItemB,18.3,false").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "3,ItemC,99.9,yes").unwrap();
        
        let records = parse_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "ItemA");
        assert_eq!(records[1].active, false);
        assert_eq!(records[2].value, 99.9);
    }
    
    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            CsvRecord { id: 1, name: "Test1".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "Test2".to_string(), value: 20.0, active: false },
            CsvRecord { id: 3, name: "Test3".to_string(), value: 30.0, active: true },
        ];
        
        let (avg, std_dev, count) = calculate_statistics(&records);
        assert_eq!(count, 3);
        assert_eq!(avg, 20.0);
        assert!(std_dev - 8.164965 < 0.0001);
    }
    
    #[test]
    fn test_invalid_format() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,ItemA,not_a_number,true").unwrap();
        
        let result = parse_csv_file(temp_file.path());
        assert!(matches!(result, Err(ParseError::ParseError(_, 1))));
    }
}