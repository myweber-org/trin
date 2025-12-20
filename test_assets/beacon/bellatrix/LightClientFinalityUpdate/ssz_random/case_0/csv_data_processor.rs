
use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Record {
            id,
            name,
            value,
            category,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        if self.category.trim().is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(())
    }

    pub fn transform(&mut self) {
        self.name = self.name.trim().to_uppercase();
        self.category = self.category.trim().to_lowercase();
        self.value = (self.value * 100.0).round() / 100.0;
    }
}

pub fn process_csv_file(input_path: &Path, output_path: &Path) -> Result<usize, Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;

    let mut processed_count = 0;

    for result in reader.deserialize() {
        let mut record: Record = result?;

        if let Err(e) = record.validate() {
            eprintln!("Validation failed for record {}: {}", record.id, e);
            continue;
        }

        record.transform();
        writer.serialize(&record)?;
        processed_count += 1;
    }

    writer.flush()?;
    Ok(processed_count)
}

pub fn filter_records_by_category(
    records: Vec<Record>,
    category_filter: &str,
) -> Vec<Record> {
    records
        .into_iter()
        .filter(|r| r.category == category_filter)
        .collect()
}

pub fn calculate_total_value(records: &[Record]) -> f64 {
    records.iter().map(|r| r.value).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "Test".to_string(), 10.5, "sample".to_string());
        assert!(valid_record.validate().is_ok());

        let invalid_record = Record::new(2, "".to_string(), -5.0, "".to_string());
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_record_transformation() {
        let mut record = Record::new(1, "  test  ".to_string(), 10.567, "  SAMPLE  ".to_string());
        record.transform();
        assert_eq!(record.name, "TEST");
        assert_eq!(record.category, "sample");
        assert_eq!(record.value, 10.57);
    }

    #[test]
    fn test_filter_records() {
        let records = vec![
            Record::new(1, "A".to_string(), 10.0, "cat1".to_string()),
            Record::new(2, "B".to_string(), 20.0, "cat2".to_string()),
            Record::new(3, "C".to_string(), 30.0, "cat1".to_string()),
        ];

        let filtered = filter_records_by_category(records, "cat1");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, 1);
        assert_eq!(filtered[1].id, 3);
    }

    #[test]
    fn test_calculate_total() {
        let records = vec![
            Record::new(1, "A".to_string(), 10.0, "cat1".to_string()),
            Record::new(2, "B".to_string(), 20.0, "cat2".to_string()),
            Record::new(3, "C".to_string(), 30.0, "cat1".to_string()),
        ];

        let total = calculate_total_value(&records);
        assert_eq!(total, 60.0);
    }

    #[test]
    fn test_csv_processing() -> Result<(), Box<dyn Error>> {
        let input_data = "id,name,value,category\n1,Test One,10.5,category1\n2,Test Two,20.75,category2";
        
        let input_file = NamedTempFile::new()?;
        std::fs::write(&input_file, input_data)?;
        
        let output_file = NamedTempFile::new()?;
        
        let processed = process_csv_file(input_file.path(), output_file.path())?;
        assert_eq!(processed, 2);
        
        Ok(())
    }
}