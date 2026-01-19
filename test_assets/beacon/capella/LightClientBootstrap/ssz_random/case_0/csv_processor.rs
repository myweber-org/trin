use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub columns: Vec<String>,
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
    expected_columns: Option<usize>,
}

impl CsvProcessor {
    pub fn new(delimiter: char) -> Self {
        CsvProcessor {
            delimiter,
            expected_columns: None,
        }
    }

    pub fn with_column_count(mut self, count: usize) -> Self {
        self.expected_columns = Some(count);
        self
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
            let line_content = line.map_err(|e| {
                CsvError::IoError(format!("Failed to read line {}: {}", line_number, e))
            })?;

            let trimmed = line_content.trim();
            if trimmed.is_empty() {
                continue;
            }

            let columns: Vec<String> = trimmed
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if let Some(expected) = self.expected_columns {
                if columns.len() != expected {
                    return Err(CsvError::ValidationError(format!(
                        "Line {}: expected {} columns, found {}",
                        line_number,
                        expected,
                        columns.len()
                    )));
                }
            }

            records.push(CsvRecord { columns });
        }

        if records.is_empty() {
            return Err(CsvError::ParseError("File contains no valid data".to_string()));
        }

        Ok(records)
    }

    pub fn extract_column(&self, records: &[CsvRecord], column_index: usize) -> Result<Vec<String>, CsvError> {
        let mut result = Vec::with_capacity(records.len());

        for (i, record) in records.iter().enumerate() {
            if column_index >= record.columns.len() {
                return Err(CsvError::ValidationError(format!(
                    "Record {}: column index {} out of bounds (max {})",
                    i + 1,
                    column_index,
                    record.columns.len() - 1
                )));
            }
            result.push(record.columns[column_index].clone());
        }

        Ok(result)
    }
}

pub fn validate_email_column(records: &[CsvRecord], email_column: usize) -> Result<(), CsvError> {
    let email_regex = regex::Regex::new(r"^[^@]+@[^@]+\.[^@]+$").unwrap();

    for (i, record) in records.iter().enumerate() {
        if email_column >= record.columns.len() {
            return Err(CsvError::ValidationError(format!(
                "Record {}: email column index out of bounds",
                i + 1
            )));
        }

        let email = &record.columns[email_column];
        if !email_regex.is_match(email) {
            return Err(CsvError::ValidationError(format!(
                "Record {}: invalid email format '{}'",
                i + 1,
                email
            )));
        }
    }

    Ok(())
}