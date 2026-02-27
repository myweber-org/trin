use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

#[derive(Debug)]
pub enum ParseError {
    IoError(String),
    ParseError(String),
    ValidationError(String),
}

impl From<std::io::Error> for ParseError {
    fn from(error: std::io::Error) -> Self {
        ParseError::IoError(error.to_string())
    }
}

pub struct CsvParser {
    delimiter: char,
    has_header: bool,
}

impl CsvParser {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvParser {
            delimiter,
            has_header,
        }
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<CsvRecord>, ParseError> {
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

    fn parse_line(&self, line: &str, line_number: usize) -> Result<CsvRecord, ParseError> {
        let parts: Vec<&str> = line.split(self.delimiter).collect();
        
        if parts.len() != 4 {
            return Err(ParseError::ParseError(
                format!("Line {}: Expected 4 fields, found {}", line_number, parts.len())
            ));
        }

        let id = parts[0].parse::<u32>()
            .map_err(|e| ParseError::ParseError(
                format!("Line {}: Invalid ID '{}': {}", line_number, parts[0], e)
            ))?;

        let name = parts[1].trim().to_string();
        if name.is_empty() {
            return Err(ParseError::ValidationError(
                format!("Line {}: Name cannot be empty", line_number)
            ));
        }

        let value = parts[2].parse::<f64>()
            .map_err(|e| ParseError::ParseError(
                format!("Line {}: Invalid value '{}': {}", line_number, parts[2], e)
            ))?;

        if value < 0.0 {
            return Err(ParseError::ValidationError(
                format!("Line {}: Value cannot be negative: {}", line_number, value)
            ));
        }

        let active = parts[3].parse::<bool>()
            .map_err(|e| ParseError::ParseError(
                format!("Line {}: Invalid boolean '{}': {}", line_number, parts[3], e)
            ))?;

        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }

    pub fn calculate_total(&self, records: &[CsvRecord]) -> f64 {
        records.iter()
            .filter(|r| r.active)
            .map(|r| r.value)
            .sum()
    }

    pub fn find_max_value(&self, records: &[CsvRecord]) -> Option<&CsvRecord> {
        records.iter()
            .filter(|r| r.active)
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,true").unwrap();
        writeln!(temp_file, "2,ItemB,20.0,false").unwrap();
        writeln!(temp_file, "3,ItemC,15.75,true").unwrap();

        let parser = CsvParser::new(',', true);
        let result = parser.parse_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "ItemA");
        assert_eq!(records[1].value, 20.0);
        assert!(!records[1].active);
    }

    #[test]
    fn test_calculate_total() {
        let records = vec![
            CsvRecord { id: 1, name: "A".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "B".to_string(), value: 20.0, active: false },
            CsvRecord { id: 3, name: "C".to_string(), value: 30.0, active: true },
        ];

        let parser = CsvParser::new(',', false);
        let total = parser.calculate_total(&records);
        
        assert_eq!(total, 40.0);
    }

    #[test]
    fn test_find_max_value() {
        let records = vec![
            CsvRecord { id: 1, name: "A".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "B".to_string(), value: 50.0, active: true },
            CsvRecord { id: 3, name: "C".to_string(), value: 30.0, active: false },
        ];

        let parser = CsvParser::new(',', false);
        let max_record = parser.find_max_value(&records);
        
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().id, 2);
    }
}