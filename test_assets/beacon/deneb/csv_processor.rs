
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

impl CsvRecord {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        Ok(())
    }
}

pub fn parse_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line = line?;
        
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() != 4 {
            return Err(format!("Invalid field count at line {}", line_number).into());
        }

        let id = fields[0].parse::<u32>()
            .map_err(|e| format!("Invalid ID at line {}: {}", line_number, e))?;
        
        let name = fields[1].trim().to_string();
        
        let value = fields[2].parse::<f64>()
            .map_err(|e| format!("Invalid value at line {}: {}", line_number, e))?;
        
        let active = fields[3].parse::<bool>()
            .map_err(|e| format!("Invalid boolean at line {}: {}", line_number, e))?;

        let record = CsvRecord { id, name, value, active };
        
        record.validate()
            .map_err(|e| format!("Validation failed at line {}: {}", line_number, e))?;
        
        records.push(record);
    }

    Ok(records)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,100.5,true").unwrap();
        writeln!(temp_file, "2,Bob,200.0,false").unwrap();
        writeln!(temp_file, "3,Charlie,300.75,true").unwrap();
        
        let records = parse_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "Alice");
        assert_eq!(records[1].value, 200.0);
        assert!(!records[1].active);
    }

    #[test]
    fn test_calculate_total_value() {
        let records = vec![
            CsvRecord { id: 1, name: "Test1".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "Test2".to_string(), value: 20.0, active: false },
            CsvRecord { id: 3, name: "Test3".to_string(), value: 30.0, active: true },
        ];
        
        let total = calculate_total_value(&records);
        assert_eq!(total, 40.0);
    }

    #[test]
    fn test_validation() {
        let valid_record = CsvRecord {
            id: 1,
            name: "Valid".to_string(),
            value: 50.0,
            active: true,
        };
        assert!(valid_record.validate().is_ok());

        let invalid_record = CsvRecord {
            id: 2,
            name: "".to_string(),
            value: -10.0,
            active: true,
        };
        assert!(invalid_record.validate().is_err());
    }
}