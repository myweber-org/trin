
use std::collections::HashSet;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: Option<f64>,
    pub category: String,
}

pub struct DataCleaner;

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner
    }

    pub fn remove_null_values(&self, records: &[DataRecord]) -> Vec<DataRecord> {
        records
            .iter()
            .filter(|record| record.value.is_some())
            .cloned()
            .collect()
    }

    pub fn deduplicate_by_id(&self, records: &[DataRecord]) -> Vec<DataRecord> {
        let mut seen_ids = HashSet::new();
        let mut deduplicated = Vec::new();

        for record in records {
            if seen_ids.insert(record.id) {
                deduplicated.push(record.clone());
            }
        }

        deduplicated
    }

    pub fn clean_data(&self, records: &[DataRecord]) -> Result<Vec<DataRecord>, Box<dyn Error>> {
        if records.is_empty() {
            return Err("Empty dataset provided".into());
        }

        let without_nulls = self.remove_null_values(records);
        let cleaned = self.deduplicate_by_id(&without_nulls);

        if cleaned.is_empty() {
            return Err("All records were filtered out during cleaning".into());
        }

        Ok(cleaned)
    }

    pub fn calculate_average(&self, records: &[DataRecord]) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = records
            .iter()
            .filter(|r| r.value.is_some())
            .collect();

        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records
            .iter()
            .map(|r| r.value.unwrap())
            .sum();

        Some(sum / valid_records.len() as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_records() -> Vec<DataRecord> {
        vec![
            DataRecord { id: 1, value: Some(10.5), category: "A".to_string() },
            DataRecord { id: 2, value: None, category: "B".to_string() },
            DataRecord { id: 1, value: Some(20.0), category: "A".to_string() },
            DataRecord { id: 3, value: Some(15.0), category: "C".to_string() },
            DataRecord { id: 4, value: None, category: "D".to_string() },
        ]
    }

    #[test]
    fn test_remove_null_values() {
        let cleaner = DataCleaner::new();
        let records = create_test_records();
        let cleaned = cleaner.remove_null_values(&records);
        
        assert_eq!(cleaned.len(), 3);
        assert!(cleaned.iter().all(|r| r.value.is_some()));
    }

    #[test]
    fn test_deduplicate_by_id() {
        let cleaner = DataCleaner::new();
        let records = create_test_records();
        let deduplicated = cleaner.deduplicate_by_id(&records);
        
        assert_eq!(deduplicated.len(), 4);
        let ids: Vec<u32> = deduplicated.iter().map(|r| r.id).collect();
        let unique_ids: HashSet<u32> = ids.iter().cloned().collect();
        assert_eq!(ids.len(), unique_ids.len());
    }

    #[test]
    fn test_clean_data() {
        let cleaner = DataCleaner::new();
        let records = create_test_records();
        let result = cleaner.clean_data(&records);
        
        assert!(result.is_ok());
        let cleaned = result.unwrap();
        assert_eq!(cleaned.len(), 2);
    }

    #[test]
    fn test_calculate_average() {
        let cleaner = DataCleaner::new();
        let records = create_test_records();
        let avg = cleaner.calculate_average(&records);
        
        assert!(avg.is_some());
        assert!((avg.unwrap() - 15.166666666666666).abs() < 0.0001);
    }
}use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(Path::new(output_path))?;
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    for result in reader.deserialize() {
        let record: Record = match result {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Skipping invalid record: {}", e);
                continue;
            }
        };

        let cleaned_record = Record {
            id: record.id,
            name: record.name.trim().to_string(),
            value: record.value.max(0.0),
            active: record.active,
        };

        writer.serialize(cleaned_record)?;
    }

    writer.flush()?;
    println!("Data cleaning completed successfully");
    Ok(())
}

fn main() {
    let input_file = "raw_data.csv";
    let output_file = "cleaned_data.csv";

    if let Err(e) = clean_csv_data(input_file, output_file) {
        eprintln!("Error processing CSV: {}", e);
        std::process::exit(1);
    }
}