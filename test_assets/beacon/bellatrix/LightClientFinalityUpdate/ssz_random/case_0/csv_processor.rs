
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
}