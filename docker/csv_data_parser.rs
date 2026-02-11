
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

impl CsvRecord {
    pub fn parse_from_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        
        if parts.len() != 4 {
            return Err("Invalid number of fields".into());
        }
        
        let id = u32::from_str(parts[0].trim())?;
        let name = parts[1].trim().to_string();
        let value = f64::from_str(parts[2].trim())?;
        let active = bool::from_str(parts[3].trim())?;
        
        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }
}

pub fn parse_csv_file(file_path: &str) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    
    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        
        match CsvRecord::parse_from_line(&line) {
            Ok(record) => records.push(record),
            Err(e) => eprintln!("Warning: Failed to parse line {}: {}", line_num + 1, e),
        }
    }
    
    Ok(records)
}

pub fn calculate_average_value(records: &[CsvRecord]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}

pub fn filter_active_records(records: &[CsvRecord]) -> Vec<&CsvRecord> {
    records.iter().filter(|r| r.active).collect()
}