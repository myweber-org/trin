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

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
    }

    pub fn transform_value(&mut self, multiplier: f64) {
        self.value *= multiplier;
    }
}

pub fn read_csv_file(file_path: &Path) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        
        if line_num == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            continue;
        }

        let id = parts[0].parse::<u32>()?;
        let name = parts[1].to_string();
        let value = parts[2].parse::<f64>()?;
        let category = parts[3].to_string();

        let record = CsvRecord::new(id, name, value, category);
        if record.is_valid() {
            records.push(record);
        }
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

pub fn process_records(records: &mut [CsvRecord], multiplier: f64) {
    for record in records.iter_mut() {
        record.transform_value(multiplier);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_record_validation() {
        let valid_record = CsvRecord::new(1, "Test".to_string(), 10.5, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = CsvRecord::new(2, "".to_string(), -5.0, "B".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_read_csv_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Item1,10.5,CategoryA").unwrap();
        writeln!(temp_file, "2,Item2,20.0,CategoryB").unwrap();

        let records = read_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "Item1");
        assert_eq!(records[1].value, 20.0);
    }

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            CsvRecord::new(1, "A".to_string(), 10.0, "X".to_string()),
            CsvRecord::new(2, "B".to_string(), 20.0, "Y".to_string()),
            CsvRecord::new(3, "C".to_string(), 30.0, "X".to_string()),
        ];

        let filtered = filter_by_category(&records, "X");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, 1);
        assert_eq!(filtered[1].id, 3);
    }

    #[test]
    fn test_calculate_total_value() {
        let records = vec![
            CsvRecord::new(1, "A".to_string(), 10.0, "X".to_string()),
            CsvRecord::new(2, "B".to_string(), 20.0, "Y".to_string()),
            CsvRecord::new(3, "C".to_string(), 30.0, "Z".to_string()),
        ];

        let total = calculate_total_value(&records);
        assert_eq!(total, 60.0);
    }

    #[test]
    fn test_process_records() {
        let mut records = vec![
            CsvRecord::new(1, "A".to_string(), 10.0, "X".to_string()),
            CsvRecord::new(2, "B".to_string(), 20.0, "Y".to_string()),
        ];

        process_records(&mut records, 2.0);
        assert_eq!(records[0].value, 20.0);
        assert_eq!(records[1].value, 40.0);
    }
}