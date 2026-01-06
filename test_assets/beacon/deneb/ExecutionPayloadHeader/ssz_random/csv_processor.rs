
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
    delimiter: char,
    has_header: bool,
}

impl Default for CsvProcessor {
    fn default() -> Self {
        Self {
            delimiter: ',',
            has_header: true,
        }
    }
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        Self {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<CsvRecord>, CsvError> {
        let file = File::open(&path).map_err(|e| {
            CsvError::IoError(format!("Failed to open file: {}", e))
        })?;

        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line = line.map_err(|e| {
                CsvError::IoError(format!("Failed to read line {}: {}", line_number, e))
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

    fn parse_line(&self, line: &str, line_number: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(self.delimiter).collect();

        if parts.len() != 4 {
            return Err(CsvError::ParseError(format!(
                "Line {}: Expected 4 fields, found {}",
                line_number,
                parts.len()
            )));
        }

        let id = parts[0].parse::<u32>().map_err(|_| {
            CsvError::ParseError(format!("Line {}: Invalid ID format", line_number))
        })?;

        let name = parts[1].trim().to_string();
        if name.is_empty() {
            return Err(CsvError::ValidationError(format!(
                "Line {}: Name cannot be empty",
                line_number
            )));
        }

        let value = parts[2].parse::<f64>().map_err(|_| {
            CsvError::ParseError(format!("Line {}: Invalid value format", line_number))
        })?;

        let active = parts[3].parse::<bool>().map_err(|_| {
            CsvError::ParseError(format!("Line {}: Invalid boolean format", line_number))
        })?;

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

        let variance: f64 = records
            .iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>()
            / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }

    pub fn filter_active_records(records: &[CsvRecord]) -> Vec<&CsvRecord> {
        records.iter().filter(|r| r.active).collect()
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
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,Test1,10.5,true").unwrap();
        writeln!(temp_file, "2,Test2,20.0,false").unwrap();

        let processor = CsvProcessor::default();
        let records = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].id, 1);
        assert_eq!(records[0].name, "Test1");
        assert_eq!(records[0].value, 10.5);
        assert!(records[0].active);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            CsvRecord {
                id: 1,
                name: "A".to_string(),
                value: 10.0,
                active: true,
            },
            CsvRecord {
                id: 2,
                name: "B".to_string(),
                value: 20.0,
                active: true,
            },
            CsvRecord {
                id: 3,
                name: "C".to_string(),
                value: 30.0,
                active: false,
            },
        ];

        let (mean, variance, std_dev) = CsvProcessor::calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }

    #[test]
    fn test_filter_active() {
        let records = vec![
            CsvRecord {
                id: 1,
                name: "A".to_string(),
                value: 10.0,
                active: true,
            },
            CsvRecord {
                id: 2,
                name: "B".to_string(),
                value: 20.0,
                active: false,
            },
            CsvRecord {
                id: 3,
                name: "C".to_string(),
                value: 30.0,
                active: true,
            },
        ];

        let active = CsvProcessor::filter_active_records(&records);
        assert_eq!(active.len(), 2);
        assert_eq!(active[0].id, 1);
        assert_eq!(active[1].id, 3);
    }
}