
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
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Self {
            id,
            name,
            value,
            category,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        if self.category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(())
    }
}

pub fn read_csv_file(file_path: &Path) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_number, line) in reader.lines().enumerate() {
        let line = line?;
        
        if line_number == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid CSV format at line {}", line_number + 1).into());
        }

        let id = parts[0].parse::<u32>()?;
        let name = parts[1].to_string();
        let value = parts[2].parse::<f64>()?;
        let category = parts[3].to_string();

        let record = CsvRecord::new(id, name, value, category);
        if let Err(e) = record.validate() {
            return Err(format!("Validation error at line {}: {}", line_number + 1, e).into());
        }

        records.push(record);
    }

    Ok(records)
}

pub fn filter_by_category(records: &[CsvRecord], category: &str) -> Vec<CsvRecord> {
    records
        .iter()
        .filter(|r| r.category == category)
        .cloned()
        .collect()
}

pub fn calculate_total_value(records: &[CsvRecord]) -> f64 {
    records.iter().map(|r| r.value).sum()
}

pub fn find_max_value_record(records: &[CsvRecord]) -> Option<&CsvRecord> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

pub fn transform_records(records: &[CsvRecord], multiplier: f64) -> Vec<CsvRecord> {
    records
        .iter()
        .map(|r| {
            let mut new_record = r.clone();
            new_record.value *= multiplier;
            new_record
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,value,category").unwrap();
        writeln!(file, "1,ItemA,100.0,Electronics").unwrap();
        writeln!(file, "2,ItemB,200.0,Books").unwrap();
        writeln!(file, "3,ItemC,150.0,Electronics").unwrap();
        file
    }

    #[test]
    fn test_read_csv_file() {
        let file = create_test_csv();
        let records = read_csv_file(file.path()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "ItemA");
        assert_eq!(records[1].value, 200.0);
    }

    #[test]
    fn test_filter_by_category() {
        let file = create_test_csv();
        let records = read_csv_file(file.path()).unwrap();
        let electronics = filter_by_category(&records, "Electronics");
        assert_eq!(electronics.len(), 2);
        assert_eq!(electronics[0].id, 1);
        assert_eq!(electronics[1].id, 3);
    }

    #[test]
    fn test_calculate_total_value() {
        let file = create_test_csv();
        let records = read_csv_file(file.path()).unwrap();
        let total = calculate_total_value(&records);
        assert_eq!(total, 450.0);
    }

    #[test]
    fn test_find_max_value_record() {
        let file = create_test_csv();
        let records = read_csv_file(file.path()).unwrap();
        let max_record = find_max_value_record(&records).unwrap();
        assert_eq!(max_record.id, 2);
        assert_eq!(max_record.value, 200.0);
    }

    #[test]
    fn test_transform_records() {
        let file = create_test_csv();
        let records = read_csv_file(file.path()).unwrap();
        let transformed = transform_records(&records, 1.5);
        assert_eq!(transformed[0].value, 150.0);
        assert_eq!(transformed[1].value, 300.0);
        assert_eq!(transformed[2].value, 225.0);
    }

    #[test]
    fn test_record_validation() {
        let valid_record = CsvRecord::new(1, "Test".to_string(), 100.0, "Category".to_string());
        assert!(valid_record.validate().is_ok());

        let invalid_record = CsvRecord::new(2, "".to_string(), -50.0, "".to_string());
        assert!(invalid_record.validate().is_err());
    }
}