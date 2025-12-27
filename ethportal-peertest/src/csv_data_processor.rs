use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl CsvRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Result<Self, String> {
        if name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        if category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }

        Ok(Self {
            id,
            name,
            value,
            category,
        })
    }

    pub fn transform_value(&mut self, multiplier: f64) {
        self.value *= multiplier;
    }
}

pub fn read_csv_file(file_path: &Path) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line_content = line?;
        
        if line_content.trim().is_empty() || line_content.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line_content.split(',').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid CSV format at line {}", line_number).into());
        }

        let id = parts[0].parse::<u32>()
            .map_err(|e| format!("Invalid ID at line {}: {}", line_number, e))?;
        
        let name = parts[1].trim().to_string();
        
        let value = parts[2].parse::<f64>()
            .map_err(|e| format!("Invalid value at line {}: {}", line_number, e))?;
        
        let category = parts[3].trim().to_string();

        match CsvRecord::new(id, name, value, category) {
            Ok(record) => records.push(record),
            Err(e) => return Err(format!("Validation error at line {}: {}", line_number, e).into()),
        }
    }

    Ok(records)
}

pub fn calculate_total_value(records: &[CsvRecord]) -> f64 {
    records.iter().map(|r| r.value).sum()
}

pub fn filter_by_category(records: Vec<CsvRecord>, category: &str) -> Vec<CsvRecord> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}

pub fn process_records(records: &mut [CsvRecord], multiplier: f64) {
    for record in records {
        record.transform_value(multiplier);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_record_creation() {
        let record = CsvRecord::new(1, "Test".to_string(), 100.0, "A".to_string());
        assert!(record.is_ok());
        
        let invalid_record = CsvRecord::new(2, "".to_string(), 50.0, "B".to_string());
        assert!(invalid_record.is_err());
    }

    #[test]
    fn test_read_csv_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Item1,100.5,CategoryA").unwrap();
        writeln!(temp_file, "2,Item2,200.0,CategoryB").unwrap();
        
        let records = read_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "Item1");
        assert_eq!(records[1].value, 200.0);
    }

    #[test]
    fn test_calculate_total_value() {
        let records = vec![
            CsvRecord::new(1, "A".to_string(), 10.0, "X".to_string()).unwrap(),
            CsvRecord::new(2, "B".to_string(), 20.0, "X".to_string()).unwrap(),
            CsvRecord::new(3, "C".to_string(), 30.0, "Y".to_string()).unwrap(),
        ];
        
        let total = calculate_total_value(&records);
        assert_eq!(total, 60.0);
    }

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            CsvRecord::new(1, "A".to_string(), 10.0, "X".to_string()).unwrap(),
            CsvRecord::new(2, "B".to_string(), 20.0, "Y".to_string()).unwrap(),
            CsvRecord::new(3, "C".to_string(), 30.0, "X".to_string()).unwrap(),
        ];
        
        let filtered = filter_by_category(records, "X");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, 1);
        assert_eq!(filtered[1].id, 3);
    }

    #[test]
    fn test_process_records() {
        let mut records = vec![
            CsvRecord::new(1, "A".to_string(), 10.0, "X".to_string()).unwrap(),
            CsvRecord::new(2, "B".to_string(), 20.0, "Y".to_string()).unwrap(),
        ];
        
        process_records(&mut records, 2.0);
        assert_eq!(records[0].value, 20.0);
        assert_eq!(records[1].value, 40.0);
    }
}