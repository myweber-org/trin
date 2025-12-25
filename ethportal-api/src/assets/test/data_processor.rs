
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { records: Vec::new() }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);

        let mut count = 0;
        for result in reader.deserialize() {
            let record: Record = result?;
            self.records.push(record);
            count += 1;
        }

        Ok(count)
    }

    pub fn save_to_csv(&self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let file = File::create(file_path)?;
        let mut writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(file);

        let mut count = 0;
        for record in &self.records {
            writer.serialize(record)?;
            count += 1;
        }

        writer.flush()?;
        Ok(count)
    }

    pub fn filter_active(&self) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.active)
            .cloned()
            .collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records
            .iter()
            .map(|r| r.value)
            .sum()
    }

    pub fn add_record(&mut self, record: Record) {
        self.records.push(record);
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }

    pub fn count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let test_record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            active: true,
        };
        
        processor.add_record(test_record.clone());
        assert_eq!(processor.count(), 1);
        
        let active_records = processor.filter_active();
        assert_eq!(active_records.len(), 1);
        
        let total = processor.calculate_total();
        assert_eq!(total, 100.0);
        
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_str().unwrap();
        
        processor.save_to_csv(file_path).unwrap();
        
        let mut new_processor = DataProcessor::new();
        let loaded = new_processor.load_from_csv(file_path).unwrap();
        assert_eq!(loaded, 1);
        assert_eq!(new_processor.count(), 1);
    }
}use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
}

pub fn process_csv(input_path: &Path, output_path: &Path) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.is_valid() {
            writer.serialize(&record)?;
        } else {
            eprintln!("Skipping invalid record: {:?}", record);
        }
    }

    writer.flush()?;
    Ok(())
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let values: Vec<f64> = records.iter().map(|r| r.value).collect();
    
    let sum: f64 = values.iter().sum();
    let count = values.len() as f64;
    let mean = if count > 0.0 { sum / count } else { 0.0 };
    
    let variance: f64 = values.iter()
        .map(|v| (v - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 42.0,
            active: true,
        };
        assert!(valid_record.is_valid());

        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            value: -5.0,
            active: false,
        };
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "B".to_string(), value: 20.0, active: true },
            Record { id: 3, name: "C".to_string(), value: 30.0, active: false },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}