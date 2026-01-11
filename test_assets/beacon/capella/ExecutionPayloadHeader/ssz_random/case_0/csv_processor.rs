
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
pub enum CsvError {
    IoError(std::io::Error),
    ParseError(String),
    ValidationError(String),
}

impl From<std::io::Error> for CsvError {
    fn from(err: std::io::Error) -> Self {
        CsvError::IoError(err)
    }
}

pub struct CsvProcessor {
    delimiter: char,
    strict_mode: bool,
}

impl Default for CsvProcessor {
    fn default() -> Self {
        CsvProcessor {
            delimiter: ',',
            strict_mode: false,
        }
    }
}

impl CsvProcessor {
    pub fn new(delimiter: char, strict_mode: bool) -> Self {
        CsvProcessor {
            delimiter,
            strict_mode,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<CsvRecord>, CsvError> {
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

            let record = self.parse_line(&line_content, line_number)?;
            records.push(record);
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_number: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(self.delimiter).collect();
        
        if parts.len() != 4 {
            return Err(CsvError::ParseError(format!(
                "Line {}: Expected 4 fields, found {}",
                line_number,
                parts.len()
            )));
        }

        let id = parts[0].parse::<u32>()
            .map_err(|e| CsvError::ParseError(format!("Line {}: Invalid ID: {}", line_number, e)))?;

        let name = parts[1].trim().to_string();
        if name.is_empty() && self.strict_mode {
            return Err(CsvError::ValidationError(format!(
                "Line {}: Name cannot be empty",
                line_number
            )));
        }

        let value = parts[2].parse::<f64>()
            .map_err(|e| CsvError::ParseError(format!("Line {}: Invalid value: {}", line_number, e)))?;

        let active = match parts[3].trim().to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(CsvError::ParseError(format!(
                "Line {}: Invalid boolean value: {}",
                line_number,
                parts[3]
            ))),
        };

        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }

    pub fn calculate_stats(records: &[CsvRecord]) -> (f64, f64, usize) {
        if records.is_empty() {
            return (0.0, 0.0, 0);
        }

        let sum: f64 = records.iter().map(|r| r.value).sum();
        let count = records.len();
        let avg = sum / count as f64;
        
        let variance: f64 = records.iter()
            .map(|r| (r.value - avg).powi(2))
            .sum::<f64>() / count as f64;
        
        (avg, variance.sqrt(), count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,42.5,true").unwrap();
        writeln!(temp_file, "2,Bob,37.8,false").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "3,Charlie,99.9,yes").unwrap();

        let processor = CsvProcessor::default();
        let records = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "Alice");
        assert_eq!(records[1].active, false);
        assert_eq!(records[2].value, 99.9);
    }

    #[test]
    fn test_stats_calculation() {
        let records = vec![
            CsvRecord { id: 1, name: "Test1".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "Test2".to_string(), value: 20.0, active: false },
            CsvRecord { id: 3, name: "Test3".to_string(), value: 30.0, active: true },
        ];

        let (avg, std_dev, count) = CsvProcessor::calculate_stats(&records);
        
        assert_eq!(avg, 20.0);
        assert!((std_dev - 8.164965).abs() < 0.0001);
        assert_eq!(count, 3);
    }
}use std::error::Error;
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
pub enum CsvError {
    IoError(String),
    ParseError(String),
    ValidationError(String),
}

impl std::fmt::Display for CsvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CsvError::IoError(msg) => write!(f, "IO error: {}", msg),
            CsvError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            CsvError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for CsvError {}

pub fn process_csv_file(file_path: &str) -> Result<Vec<CsvRecord>, CsvError> {
    let path = Path::new(file_path);
    let file = File::open(path).map_err(|e| CsvError::IoError(e.to_string()))?;
    let reader = BufReader::new(file);
    
    let mut records = Vec::new();
    let mut line_number = 0;
    
    for line_result in reader.lines() {
        line_number += 1;
        let line = line_result.map_err(|e| CsvError::IoError(e.to_string()))?;
        
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        
        let fields: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        
        if fields.len() != 4 {
            return Err(CsvError::ParseError(
                format!("Line {}: expected 4 fields, found {}", line_number, fields.len())
            ));
        }
        
        let id = fields[0].parse::<u32>()
            .map_err(|_| CsvError::ParseError(
                format!("Line {}: invalid ID '{}'", line_number, fields[0])
            ))?;
        
        let name = fields[1].to_string();
        if name.is_empty() {
            return Err(CsvError::ValidationError(
                format!("Line {}: name cannot be empty", line_number)
            ));
        }
        
        let value = fields[2].parse::<f64>()
            .map_err(|_| CsvError::ParseError(
                format!("Line {}: invalid value '{}'", line_number, fields[2])
            ))?;
        
        let active = match fields[3].to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(CsvError::ParseError(
                format!("Line {}: invalid boolean '{}'", line_number, fields[3])
            )),
        };
        
        if value < 0.0 {
            return Err(CsvError::ValidationError(
                format!("Line {}: value cannot be negative", line_number)
            ));
        }
        
        records.push(CsvRecord {
            id,
            name,
            value,
            active,
        });
    }
    
    if records.is_empty() {
        return Err(CsvError::ValidationError("File contains no valid records".to_string()));
    }
    
    Ok(records)
}

pub fn calculate_statistics(records: &[CsvRecord]) -> (f64, f64, f64) {
    let active_records: Vec<&CsvRecord> = records.iter()
        .filter(|r| r.active)
        .collect();
    
    if active_records.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = active_records.iter().map(|r| r.value).sum();
    let count = active_records.len() as f64;
    let average = sum / count;
    
    let variance: f64 = active_records.iter()
        .map(|r| (r.value - average).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (sum, average, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_valid_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,42.5,true").unwrap();
        writeln!(temp_file, "2,Bob,33.7,false").unwrap();
        writeln!(temp_file, "3,Charlie,15.2,true").unwrap();
        
        let records = process_csv_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "Alice");
        assert_eq!(records[1].value, 33.7);
        assert!(!records[1].active);
    }
    
    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            CsvRecord { id: 1, name: "Test1".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "Test2".to_string(), value: 20.0, active: true },
            CsvRecord { id: 3, name: "Test3".to_string(), value: 30.0, active: false },
        ];
        
        let (sum, avg, std_dev) = calculate_statistics(&records);
        assert_eq!(sum, 30.0);
        assert_eq!(avg, 15.0);
        assert!(std_dev > 7.07 && std_dev < 7.08);
    }
    
    #[test]
    fn test_invalid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,invalid,true").unwrap();
        
        let result = process_csv_file(temp_file.path().to_str().unwrap());
        assert!(matches!(result, Err(CsvError::ParseError(_))));
    }
}