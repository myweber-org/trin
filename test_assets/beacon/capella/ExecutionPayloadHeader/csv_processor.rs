
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

pub fn parse_csv<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    
    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        
        let parts: Vec<&str> = line.split(',').collect();
        
        if parts.len() != 4 {
            return Err(format!("Invalid column count at line {}", line_num + 1).into());
        }
        
        let id = parts[0].parse::<u32>()
            .map_err(|e| format!("Invalid ID at line {}: {}", line_num + 1, e))?;
        
        let name = parts[1].trim().to_string();
        
        if name.is_empty() {
            return Err(format!("Empty name at line {}", line_num + 1).into());
        }
        
        let value = parts[2].parse::<f64>()
            .map_err(|e| format!("Invalid value at line {}: {}", line_num + 1, e))?;
        
        let active = match parts[3].trim().to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(format!("Invalid boolean at line {}", line_num + 1).into()),
        };
        
        records.push(Record {
            id,
            name,
            value,
            active,
        });
    }
    
    Ok(records)
}

pub fn validate_records(records: &[Record]) -> Result<(), Box<dyn Error>> {
    let mut seen_ids = std::collections::HashSet::new();
    
    for record in records {
        if !seen_ids.insert(record.id) {
            return Err(format!("Duplicate ID found: {}", record.id).into());
        }
        
        if record.value < 0.0 {
            return Err(format!("Negative value for ID {}: {}", record.id, record.value).into());
        }
        
        if record.name.len() > 100 {
            return Err(format!("Name too long for ID {}: {}", record.id, record.name.len()).into());
        }
    }
    
    Ok(())
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let active_records: Vec<&Record> = records.iter()
        .filter(|r| r.active)
        .collect();
    
    let total: f64 = records.iter().map(|r| r.value).sum();
    let active_total: f64 = active_records.iter().map(|r| r.value).sum();
    let average = total / records.len() as f64;
    
    (total, active_total, average)
}