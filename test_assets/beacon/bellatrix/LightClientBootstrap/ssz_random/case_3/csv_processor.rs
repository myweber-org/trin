
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

            if self.has_header && line_number == 1 {
                continue;
            }

            if line.trim().is_empty() {
                continue;
            }

            let record = self.parse_line(&line, line_number)?;
            records.push(record);
        }

        if records.is_empty() {
            return Err(CsvError::ValidationError(
                "No valid records found in CSV file".to_string(),
            ));
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_number: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(self.delimiter).collect();

        if parts.len() != 4 {
            return Err(CsvError::ParseError(format!(
                "Line {}: Expected 4 columns, found {}",
                line_number,
                parts.len()
            )));
        }

        let id = parts[0]
            .parse::<u32>()
            .map_err(|e| CsvError::ParseError(format!("Line {}: Invalid ID: {}", line_number, e)))?;

        let name = parts[1].trim().to_string();
        if name.is_empty() {
            return Err(CsvError::ValidationError(format!(
                "Line {}: Name cannot be empty",
                line_number
            )));
        }

        let value = parts[2]
            .parse::<f64>()
            .map_err(|e| CsvError::ParseError(format!("Line {}: Invalid value: {}", line_number, e)))?;

        if value < 0.0 {
            return Err(CsvError::ValidationError(format!(
                "Line {}: Value cannot be negative",
                line_number
            )));
        }

        let active = parts[3]
            .parse::<bool>()
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

        let variance: f64 = records
            .iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>()
            / count;

        let std_dev = variance.sqrt();

        (sum, mean, std_dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let csv_data = "id,name,value,active\n1,Test1,10.5,true\n2,Test2,20.0,false\n";
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();

        let processor = CsvProcessor::default();
        let records = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "Test1");
        assert_eq!(records[1].value, 20.0);
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
                active: false,
            },
            CsvRecord {
                id: 3,
                name: "C".to_string(),
                value: 30.0,
                active: true,
            },
        ];

        let (sum, mean, std_dev) = CsvProcessor::calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(mean, 20.0);
        assert!((std_dev - 8.164965).abs() < 0.0001);
    }

    #[test]
    fn test_invalid_csv() {
        let csv_data = "id,name,value\n1,Test1,10.5\n";
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();

        let processor = CsvProcessor::default();
        let result = processor.process_file(temp_file.path());
        assert!(result.is_err());
    }
}