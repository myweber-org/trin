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
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), CsvError> {
        let file = File::open(&path).map_err(|e| {
            CsvError::IoError(format!("Failed to open file {}: {}", path.as_ref().display(), e))
        })?;

        let reader = BufReader::new(file);
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line = line.map_err(|e| {
                CsvError::IoError(format!("Failed to read line {}: {}", line_number, e))
            })?;

            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let record = self.parse_line(&line, line_number)?;
            self.validate_record(&record, line_number)?;
            self.records.push(record);
        }

        Ok(())
    }

    fn parse_line(&self, line: &str, line_number: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

        if parts.len() != 4 {
            return Err(CsvError::ParseError(format!(
                "Line {}: Expected 4 fields, found {}",
                line_number,
                parts.len()
            )));
        }

        let id = parts[0].parse::<u32>().map_err(|e| {
            CsvError::ParseError(format!("Line {}: Invalid ID format: {}", line_number, e))
        })?;

        let name = parts[1].to_string();
        if name.is_empty() {
            return Err(CsvError::ParseError(format!(
                "Line {}: Name cannot be empty",
                line_number
            )));
        }

        let value = parts[2].parse::<f64>().map_err(|e| {
            CsvError::ParseError(format!("Line {}: Invalid value format: {}", line_number, e))
        })?;

        let active = parts[3].parse::<bool>().map_err(|e| {
            CsvError::ParseError(format!("Line {}: Invalid boolean format: {}", line_number, e))
        })?;

        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }

    fn validate_record(&self, record: &CsvRecord, line_number: usize) -> Result<(), CsvError> {
        if record.id == 0 {
            return Err(CsvError::ValidationError(format!(
                "Line {}: ID cannot be zero",
                line_number
            )));
        }

        if record.value < 0.0 {
            return Err(CsvError::ValidationError(format!(
                "Line {}: Value cannot be negative",
                line_number
            )));
        }

        Ok(())
    }

    pub fn get_active_records(&self) -> Vec<&CsvRecord> {
        self.records.iter().filter(|r| r.active).collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn find_by_name(&self, name: &str) -> Option<&CsvRecord> {
        self.records.iter().find(|r| r.name == name)
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_parsing() {
        let mut csv_data = NamedTempFile::new().unwrap();
        writeln!(csv_data, "1,Item One,100.5,true").unwrap();
        writeln!(csv_data, "2,Item Two,200.0,false").unwrap();
        writeln!(csv_data, "3,Item Three,300.75,true").unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(csv_data.path());
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 3);
    }

    #[test]
    fn test_invalid_csv() {
        let mut csv_data = NamedTempFile::new().unwrap();
        writeln!(csv_data, "invalid,data,here").unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(csv_data.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_active_records() {
        let mut processor = CsvProcessor::new();
        processor.records.push(CsvRecord {
            id: 1,
            name: "Test1".to_string(),
            value: 100.0,
            active: true,
        });
        processor.records.push(CsvRecord {
            id: 2,
            name: "Test2".to_string(),
            value: 200.0,
            active: false,
        });

        let active = processor.get_active_records();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, 1);
    }

    #[test]
    fn test_total_value() {
        let mut processor = CsvProcessor::new();
        processor.records.push(CsvRecord {
            id: 1,
            name: "Test1".to_string(),
            value: 100.0,
            active: true,
        });
        processor.records.push(CsvRecord {
            id: 2,
            name: "Test2".to_string(),
            value: 200.0,
            active: false,
        });

        assert_eq!(processor.calculate_total_value(), 300.0);
    }
}