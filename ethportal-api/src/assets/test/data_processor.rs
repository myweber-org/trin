
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
}