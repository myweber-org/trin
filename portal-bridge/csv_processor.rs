
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

#[derive(Debug)]
pub enum ParseError {
    IoError(String),
    InvalidFormat(String),
    InvalidData(String),
}

impl From<std::io::Error> for ParseError {
    fn from(error: std::io::Error) -> Self {
        ParseError::IoError(error.to_string())
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

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Record>, ParseError> {
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

    fn parse_line(&self, line: &str, line_number: usize) -> Result<Record, ParseError> {
        let parts: Vec<&str> = line.split(self.delimiter).collect();

        if parts.len() != 4 {
            return Err(ParseError::InvalidFormat(format!(
                "Line {}: Expected 4 fields, found {}",
                line_number,
                parts.len()
            )));
        }

        let id = parts[0]
            .parse::<u32>()
            .map_err(|_| ParseError::InvalidData(format!("Line {}: Invalid ID format", line_number)))?;

        let name = parts[1].trim().to_string();
        if name.is_empty() && self.strict_mode {
            return Err(ParseError::InvalidData(format!("Line {}: Name cannot be empty", line_number)));
        }

        let value = parts[2]
            .parse::<f64>()
            .map_err(|_| ParseError::InvalidData(format!("Line {}: Invalid value format", line_number)))?;

        let active = parts[3]
            .parse::<bool>()
            .map_err(|_| ParseError::InvalidData(format!("Line {}: Invalid boolean format", line_number)))?;

        Ok(Record {
            id,
            name,
            value,
            active,
        })
    }

    pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
        if records.is_empty() {
            return (0.0, 0.0, 0);
        }

        let sum: f64 = records.iter().map(|r| r.value).sum();
        let count = records.len();
        let average = sum / count as f64;

        let variance: f64 = records
            .iter()
            .map(|r| (r.value - average).powi(2))
            .sum::<f64>()
            / count as f64;

        (average, variance, count)
    }
}

impl Default for CsvProcessor {
    fn default() -> Self {
        Self::new()
    }
}