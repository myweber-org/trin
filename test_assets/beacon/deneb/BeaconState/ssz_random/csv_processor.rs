use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
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
    records: Vec<Record>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), CsvError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            
            let record = self.parse_line(&line, line_num + 1)?;
            self.validate_record(&record, line_num + 1)?;
            self.records.push(record);
        }
        
        Ok(())
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<Record, CsvError> {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        
        if parts.len() != 3 {
            return Err(CsvError::ParseError(
                format!("Line {}: Expected 3 columns, found {}", line_num, parts.len())
            ));
        }
        
        let id = parts[0].parse::<u32>()
            .map_err(|e| CsvError::ParseError(
                format!("Line {}: Invalid ID format: {}", line_num, e)
            ))?;
        
        let name = parts[1].to_string();
        
        let value = parts[2].parse::<f64>()
            .map_err(|e| CsvError::ParseError(
                format!("Line {}: Invalid value format: {}", line_num, e)
            ))?;
        
        Ok(Record { id, name, value })
    }

    fn validate_record(&self, record: &Record, line_num: usize) -> Result<(), CsvError> {
        if record.name.is_empty() {
            return Err(CsvError::ValidationError(
                format!("Line {}: Name cannot be empty", line_num)
            ));
        }
        
        if record.value < 0.0 {
            return Err(CsvError::ValidationError(
                format!("Line {}: Value cannot be negative", line_num)
            ));
        }
        
        Ok(())
    }

    pub fn total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            None
        } else {
            Some(self.total_value() / self.records.len() as f64)
        }
    }

    pub fn find_by_id(&self, id: u32) -> Option<&Record> {
        self.records.iter().find(|r| r.id == id)
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

impl Default for CsvProcessor {
    fn default() -> Self {
        Self::new()
    }
}