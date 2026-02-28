
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

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
    valid_count: usize,
    invalid_count: usize,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
            valid_count: 0,
            invalid_count: 0,
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), CsvError> {
        let file = File::open(path.as_ref())
            .map_err(|e| CsvError::IoError(format!("Failed to open file: {}", e)))?;
        
        let reader = BufReader::new(file);
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| CsvError::IoError(format!("Failed to read line: {}", e)))?;
            
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            
            match self.parse_line(&line) {
                Ok(record) => {
                    self.records.push(record);
                    self.valid_count += 1;
                }
                Err(e) => {
                    eprintln!("Line {}: {}", line_num + 1, e);
                    self.invalid_count += 1;
                }
            }
        }
        
        Ok(())
    }

    fn parse_line(&self, line: &str) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        
        if parts.len() != 4 {
            return Err(CsvError::ParseError(
                format!("Expected 4 columns, found {}", parts.len())
            ));
        }
        
        let id = parts[0].parse::<u32>()
            .map_err(|e| CsvError::ParseError(format!("Invalid ID: {}", e)))?;
        
        let name = parts[1].to_string();
        if name.is_empty() {
            return Err(CsvError::ValidationError("Name cannot be empty".to_string()));
        }
        
        let value = parts[2].parse::<f64>()
            .map_err(|e| CsvError::ParseError(format!("Invalid value: {}", e)))?;
        
        if value < 0.0 {
            return Err(CsvError::ValidationError(
                format!("Value cannot be negative: {}", value)
            ));
        }
        
        let active = match parts[3].to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(CsvError::ParseError(
                format!("Invalid boolean value: {}", parts[3])
            )),
        };
        
        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter()
            .filter(|r| r.active)
            .map(|r| r.value)
            .sum()
    }

    pub fn get_stats(&self) -> (usize, usize, f64) {
        (
            self.valid_count,
            self.invalid_count,
            self.calculate_total()
        )
    }

    pub fn filter_by_value(&self, threshold: f64) -> Vec<&CsvRecord> {
        self.records.iter()
            .filter(|r| r.value >= threshold && r.active)
            .collect()
    }
}

impl Default for CsvProcessor {
    fn default() -> Self {
        Self::new()
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

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<CsvRecord>, CsvError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            let line = line?;
            line_number += 1;

            if line_number == 1 && self.has_header {
                continue;
            }

            if line.trim().is_empty() {
                continue;
            }

            let record = self.parse_line(&line, line_number)?;
            records.push(record);
        }

        if records.is_empty() {
            return Err(CsvError::ValidationError("No valid records found".to_string()));
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_number: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(self.delimiter).collect();
        
        if parts.len() != 4 {
            return Err(CsvError::ParseError(
                format!("Line {}: Expected 4 columns, found {}", line_number, parts.len())
            ));
        }

        let id = parts[0].parse::<u32>()
            .map_err(|e| CsvError::ParseError(
                format!("Line {}: Invalid ID '{}': {}", line_number, parts[0], e)
            ))?;

        let name = parts[1].trim().to_string();
        if name.is_empty() {
            return Err(CsvError::ValidationError(
                format!("Line {}: Name cannot be empty", line_number)
            ));
        }

        let value = parts[2].parse::<f64>()
            .map_err(|e| CsvError::ParseError(
                format!("Line {}: Invalid value '{}': {}", line_number, parts[2], e)
            ))?;

        if value < 0.0 {
            return Err(CsvError::ValidationError(
                format!("Line {}: Value cannot be negative: {}", line_number, value)
            ));
        }

        let active = parts[3].parse::<bool>()
            .map_err(|e| CsvError::ParseError(
                format!("Line {}: Invalid boolean '{}': {}", line_number, parts[3], e)
            ))?;

        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let csv_data = "id,name,value,active\n1,ItemA,10.5,true\n2,ItemB,20.0,false\n";
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();

        let processor = CsvProcessor::default();
        let records = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "ItemA");
        assert_eq!(records[1].value, 20.0);

        let (mean, variance, std_dev) = CsvProcessor::calculate_statistics(&records);
        assert!((mean - 15.25).abs() < 0.001);
        assert!((variance - 22.5625).abs() < 0.001);
        assert!((std_dev - 4.75).abs() < 0.001);
    }

    #[test]
    fn test_invalid_data() {
        let csv_data = "id,name,value,active\n1,,10.5,true\n";
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();

        let processor = CsvProcessor::default();
        let result = processor.process_file(temp_file.path());
        assert!(matches!(result, Err(CsvError::ValidationError(_))));
    }
}