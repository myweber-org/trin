
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

pub fn parse_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<CsvRecord>, CsvError> {
    let file = File::open(&path).map_err(|e| {
        CsvError::IoError(format!("Failed to open file {}: {}", path.as_ref().display(), e))
    })?;

    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line = line.map_err(|e| {
            CsvError::IoError(format!("Failed to read line {}: {}", line_number, e))
        })?;

        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let record = parse_csv_line(&line, line_number)?;
        records.push(record);
    }

    validate_records(&records)?;
    Ok(records)
}

fn parse_csv_line(line: &str, line_number: usize) -> Result<CsvRecord, CsvError> {
    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
    
    if parts.len() != 4 {
        return Err(CsvError::ParseError(format!(
            "Line {}: Expected 4 fields, found {}",
            line_number,
            parts.len()
        )));
    }

    let id = parts[0].parse::<u32>().map_err(|_| {
        CsvError::ParseError(format!("Line {}: Invalid ID format '{}'", line_number, parts[0]))
    })?;

    let name = parts[1].to_string();
    if name.is_empty() {
        return Err(CsvError::ValidationError(format!(
            "Line {}: Name cannot be empty",
            line_number
        )));
    }

    let value = parts[2].parse::<f64>().map_err(|_| {
        CsvError::ParseError(format!("Line {}: Invalid value format '{}'", line_number, parts[2]))
    })?;

    let active = match parts[3].to_lowercase().as_str() {
        "true" | "1" | "yes" => true,
        "false" | "0" | "no" => false,
        _ => return Err(CsvError::ParseError(format!(
            "Line {}: Invalid boolean value '{}'",
            line_number, parts[3]
        ))),
    };

    Ok(CsvRecord {
        id,
        name,
        value,
        active,
    })
}

fn validate_records(records: &[CsvRecord]) -> Result<(), CsvError> {
    let mut seen_ids = std::collections::HashSet::new();
    
    for record in records {
        if !seen_ids.insert(record.id) {
            return Err(CsvError::ValidationError(format!(
                "Duplicate ID found: {}",
                record.id
            )));
        }

        if record.value < 0.0 {
            return Err(CsvError::ValidationError(format!(
                "Record ID {} has negative value: {}",
                record.id, record.value
            )));
        }
    }

    Ok(())
}

pub fn calculate_total_value(records: &[CsvRecord]) -> f64 {
    records.iter()
        .filter(|r| r.active)
        .map(|r| r.value)
        .sum()
}

pub fn find_max_value_record(records: &[CsvRecord]) -> Option<&CsvRecord> {
    records.iter()
        .filter(|r| r.active)
        .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}