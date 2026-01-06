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

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            delimiter: ',',
            strict_mode: false,
        }
    }

    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
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
        if name.is_empty() {
            return Err(CsvError::ValidationError(format!(
                "Line {}: Name cannot be empty",
                line_number
            )));
        }

        let value = parts[2].parse::<f64>()
            .map_err(|e| CsvError::ParseError(format!("Line {}: Invalid value: {}", line_number, e)))?;

        if self.strict_mode && value < 0.0 {
            return Err(CsvError::ValidationError(format!(
                "Line {}: Value cannot be negative in strict mode",
                line_number
            )));
        }

        let active = parts[3].parse::<bool>()
            .map_err(|e| CsvError::ParseError(format!("Line {}: Invalid boolean: {}", line_number, e)))?;

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

impl Default for CsvProcessor {
    fn default() -> Self {
        Self::new()
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
        writeln!(temp_file, "2,Bob,-15.0,false").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "3,Charlie,100.0,true").unwrap();

        let processor = CsvProcessor::new();
        let records = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "Alice");
        assert_eq!(records[1].id, 2);
        assert!(!records[1].active);
    }

    #[test]
    fn test_strict_mode() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Test,-10.0,true").unwrap();

        let processor = CsvProcessor::new().strict_mode(true);
        let result = processor.process_file(temp_file.path());

        assert!(matches!(result, Err(CsvError::ValidationError(_))));
    }

    #[test]
    fn test_statistics() {
        let records = vec![
            CsvRecord { id: 1, name: "A".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "B".to_string(), value: 20.0, active: false },
            CsvRecord { id: 3, name: "C".to_string(), value: 30.0, active: true },
        ];

        let (mean, variance, std_dev) = CsvProcessor::calculate_statistics(&records);
        
        assert!((mean - 20.0).abs() < 1e-10);
        assert!((variance - 66.6666666667).abs() < 1e-10);
        assert!((std_dev - 8.16496580928).abs() < 1e-10);
    }
}