use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
enum CsvError {
    IoError(std::io::Error),
    ParseError(String, usize),
    InvalidHeader(String),
}

impl fmt::Display for CsvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CsvError::IoError(e) => write!(f, "IO error: {}", e),
            CsvError::ParseError(msg, line) => write!(f, "Parse error at line {}: {}", line, msg),
            CsvError::InvalidHeader(msg) => write!(f, "Invalid header: {}", msg),
        }
    }
}

impl Error for CsvError {}

impl From<std::io::Error> for CsvError {
    fn from(error: std::io::Error) -> Self {
        CsvError::IoError(error)
    }
}

struct CsvProcessor {
    headers: Vec<String>,
    data: Vec<Vec<String>>,
}

impl CsvProcessor {
    fn from_file(path: &str) -> Result<Self, CsvError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines().enumerate();

        let headers = match lines.next() {
            Some((_, Ok(line))) => Self::parse_line(&line, 1)?,
            Some((_, Err(e))) => return Err(CsvError::IoError(e)),
            None => return Err(CsvError::InvalidHeader("Empty file".to_string())),
        };

        if headers.is_empty() {
            return Err(CsvError::InvalidHeader("No headers found".to_string()));
        }

        let mut data = Vec::new();
        for (idx, line_result) in lines {
            let line = line_result?;
            let row = Self::parse_line(&line, idx + 2)?;
            
            if row.len() != headers.len() {
                return Err(CsvError::ParseError(
                    format!("Expected {} columns, found {}", headers.len(), row.len()),
                    idx + 2,
                ));
            }
            
            data.push(row);
        }

        Ok(CsvProcessor { headers, data })
    }

    fn parse_line(line: &str, line_number: usize) -> Result<Vec<String>, CsvError> {
        let mut result = Vec::new();
        let mut current_field = String::new();
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '"' => {
                    if in_quotes && chars.peek() == Some(&'"') {
                        current_field.push('"');
                        chars.next();
                    } else {
                        in_quotes = !in_quotes;
                    }
                }
                ',' if !in_quotes => {
                    result.push(current_field.trim().to_string());
                    current_field.clear();
                }
                _ => current_field.push(ch),
            }
        }

        result.push(current_field.trim().to_string());

        if in_quotes {
            return Err(CsvError::ParseError(
                "Unclosed quotation mark".to_string(),
                line_number,
            ));
        }

        Ok(result)
    }

    fn get_column(&self, column_name: &str) -> Result<Vec<&str>, CsvError> {
        let idx = self.headers
            .iter()
            .position(|h| h == column_name)
            .ok_or_else(|| CsvError::InvalidHeader(format!("Column '{}' not found", column_name)))?;

        Ok(self.data
            .iter()
            .map(|row| row[idx].as_str())
            .collect())
    }

    fn validate_numeric_column(&self, column_name: &str) -> Result<Vec<f64>, CsvError> {
        let values = self.get_column(column_name)?;
        let mut numeric_values = Vec::new();

        for (idx, value) in values.iter().enumerate() {
            match value.parse::<f64>() {
                Ok(num) => numeric_values.push(num),
                Err(_) => return Err(CsvError::ParseError(
                    format!("Invalid numeric value '{}'", value),
                    idx + 2,
                )),
            }
        }

        Ok(numeric_values)
    }

    fn summary(&self) -> String {
        format!(
            "CSV Summary:\n  Columns: {}\n  Rows: {}\n  Headers: {:?}",
            self.headers.len(),
            self.data.len(),
            self.headers
        )
    }
}

fn process_csv_file() -> Result<(), CsvError> {
    let processor = CsvProcessor::from_file("data.csv")?;
    
    println!("{}", processor.summary());
    
    match processor.validate_numeric_column("price") {
        Ok(prices) => {
            let avg = prices.iter().sum::<f64>() / prices.len() as f64;
            println!("Average price: {:.2}", avg);
        }
        Err(e) => println!("Price validation failed: {}", e),
    }
    
    Ok(())
}

fn main() {
    if let Err(e) = process_csv_file() {
        eprintln!("Error processing CSV: {}", e);
        std::process::exit(1);
    }
}
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
            CsvError::IoError(msg) => write!(f, "IO Error: {}", msg),
            CsvError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
            CsvError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
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
            CsvError::IoError(format!("Failed to open file: {}", e))
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
            return Err(CsvError::ParseError(
                format!("Line {}: Expected 4 columns, found {}", line_number, parts.len())
            ));
        }

        let id = parts[0].parse::<u32>().map_err(|_| {
            CsvError::ParseError(format!("Line {}: Invalid ID format '{}'", line_number, parts[0]))
        })?;

        let name = parts[1].to_string();
        
        let value = parts[2].parse::<f64>().map_err(|_| {
            CsvError::ParseError(format!("Line {}: Invalid value format '{}'", line_number, parts[2]))
        })?;

        let active = match parts[3].to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(CsvError::ParseError(
                format!("Line {}: Invalid boolean format '{}'", line_number, parts[3])
            )),
        };

        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }

    fn validate_record(&self, record: &CsvRecord, line_number: usize) -> Result<(), CsvError> {
        if record.name.is_empty() {
            return Err(CsvError::ValidationError(
                format!("Line {}: Name cannot be empty", line_number)
            ));
        }

        if record.value < 0.0 {
            return Err(CsvError::ValidationError(
                format!("Line {}: Value cannot be negative", line_number)
            ));
        }

        Ok(())
    }

    pub fn get_records(&self) -> &[CsvRecord] {
        &self.records
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter()
            .filter(|r| r.active)
            .map(|r| r.value)
            .sum()
    }

    pub fn find_by_id(&self, id: u32) -> Option<&CsvRecord> {
        self.records.iter().find(|r| r.id == id)
    }

    pub fn get_active_records(&self) -> Vec<&CsvRecord> {
        self.records.iter()
            .filter(|r| r.active)
            .collect()
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
        writeln!(temp_file, "1,ItemA,10.5,true").unwrap();
        writeln!(temp_file, "2,ItemB,20.0,false").unwrap();
        writeln!(temp_file, "3,ItemC,15.75,true").unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);
        assert_eq!(processor.calculate_total(), 26.25);
    }

    #[test]
    fn test_invalid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "invalid,data,here").unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(temp_file.path());
        
        assert!(result.is_err());
    }
}