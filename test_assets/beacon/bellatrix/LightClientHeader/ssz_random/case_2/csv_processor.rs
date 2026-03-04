use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
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

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_number: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(self.delimiter).collect();

        if parts.len() != 3 {
            return Err(CsvError::ParseError(format!(
                "Line {}: Expected 3 columns, found {}",
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
                "Line {}: Value cannot be negative: {}",
                line_number, value
            )));
        }

        Ok(CsvRecord { id, name, value })
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let csv_content = "id,name,value\n1,Test1,10.5\n2,Test2,20.0\n3,Test3,15.75";
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_content).unwrap();

        let processor = CsvProcessor::new(',', true);
        let records = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "Test1");
        assert_eq!(records[1].value, 20.0);

        let (mean, variance, std_dev) = CsvProcessor::calculate_statistics(&records);
        assert!((mean - 15.416666).abs() < 0.001);
        assert!((variance - 22.743055).abs() < 0.001);
        assert!((std_dev - 4.768968).abs() < 0.001);
    }

    #[test]
    fn test_invalid_csv() {
        let csv_content = "id,name,value\n1,Test1,invalid\n2,Test2,20.0";
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_content).unwrap();

        let processor = CsvProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        assert!(result.is_err());
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
}

#[derive(Debug)]
pub enum CsvError {
    IoError(std::io::Error),
    ParseError(String),
    InvalidHeader,
    MissingColumn,
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

        let mut lines = reader.lines().enumerate();

        if self.has_header {
            if let Some((_, header_result)) = lines.next() {
                let header = header_result?;
                self.validate_header(&header)?;
            }
        }

        for (line_num, line_result) in lines {
            let line = line_result?;
            match self.parse_line(&line, line_num + 1) {
                Ok(record) => records.push(record),
                Err(e) => eprintln!("Warning: Skipping line {}: {}", line_num + 1, e),
            }
        }

        Ok(records)
    }

    fn validate_header(&self, header: &str) -> Result<(), CsvError> {
        let columns: Vec<&str> = header.split(self.delimiter).collect();
        if columns.len() < 3 {
            return Err(CsvError::InvalidHeader);
        }

        let expected = ["id", "name", "value"];
        for (i, &col) in expected.iter().enumerate() {
            if columns.get(i).map(|&s| s.trim()) != Some(col) {
                return Err(CsvError::InvalidHeader);
            }
        }

        Ok(())
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(self.delimiter).collect();
        
        if parts.len() < 3 {
            return Err(CsvError::MissingColumn);
        }

        let id_str = parts[0].trim();
        let name = parts[1].trim().to_string();
        let value_str = parts[2].trim();

        let id = id_str.parse::<u32>()
            .map_err(|e| CsvError::ParseError(format!("Line {}: Invalid ID '{}': {}", line_num, id_str, e)))?;

        let value = value_str.parse::<f64>()
            .map_err(|e| CsvError::ParseError(format!("Line {}: Invalid value '{}': {}", line_num, value_str, e)))?;

        Ok(CsvRecord { id, name, value })
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

impl std::fmt::Display for CsvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CsvError::IoError(e) => write!(f, "IO error: {}", e),
            CsvError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            CsvError::InvalidHeader => write!(f, "Invalid CSV header format"),
            CsvError::MissingColumn => write!(f, "Missing required columns"),
        }
    }
}

impl Error for CsvError {}