use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use serde::{Deserialize, Serialize};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
    pub timestamp: String,
}

impl Record {
    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() 
            && self.value >= 0.0 
            && !self.category.is_empty()
            && !self.timestamp.is_empty()
    }
}

pub struct DataProcessor {
    pub records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        let mut count = 0;
        for result in csv_reader.deserialize() {
            let record: Record = result?;
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn save_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);

        let mut count = 0;
        for record in &self.records {
            csv_writer.serialize(record)?;
            count += 1;
        }

        csv_writer.flush()?;
        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_max_value_record(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn sort_by_value(&mut self, ascending: bool) {
        if ascending {
            self.records.sort_by(|a, b| {
                a.value.partial_cmp(&b.value).unwrap()
            });
        } else {
            self.records.sort_by(|a, b| {
                b.value.partial_cmp(&a.value).unwrap()
            });
        }
    }

    pub fn add_record(&mut self, record: Record) -> Result<(), &'static str> {
        if record.is_valid() {
            self.records.push(record);
            Ok(())
        } else {
            Err("Invalid record data")
        }
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
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
            value: 10.5,
            category: "A".to_string(),
            timestamp: "2023-01-01".to_string(),
        };
        assert!(valid_record.is_valid());

        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            value: -5.0,
            category: "B".to_string(),
            timestamp: "2023-01-01".to_string(),
        };
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();
        
        let record1 = Record {
            id: 1,
            name: "Item1".to_string(),
            value: 100.0,
            category: "Electronics".to_string(),
            timestamp: "2023-01-01".to_string(),
        };

        let record2 = Record {
            id: 2,
            name: "Item2".to_string(),
            value: 50.0,
            category: "Books".to_string(),
            timestamp: "2023-01-02".to_string(),
        };

        assert!(processor.add_record(record1.clone()).is_ok());
        assert!(processor.add_record(record2.clone()).is_ok());

        assert_eq!(processor.records.len(), 2);
        
        let avg = processor.calculate_average_value();
        assert_eq!(avg, Some(75.0));

        let max_record = processor.find_max_value_record();
        assert_eq!(max_record.unwrap().id, 1);

        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 1);
        assert_eq!(electronics[0].id, 1);

        processor.sort_by_value(true);
        assert_eq!(processor.records[0].id, 2);
        assert_eq!(processor.records[1].id, 1);

        processor.clear();
        assert!(processor.records.is_empty());
    }

    #[test]
    fn test_csv_operations() {
        let mut processor = DataProcessor::new();
        
        let record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 99.9,
            category: "Test".to_string(),
            timestamp: "2023-01-01".to_string(),
        };

        processor.add_record(record).unwrap();

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let save_result = processor.save_to_csv(path);
        assert!(save_result.is_ok());
        assert_eq!(save_result.unwrap(), 1);

        let mut new_processor = DataProcessor::new();
        let load_result = new_processor.load_from_csv(path);
        assert!(load_result.is_ok());
        assert_eq!(load_result.unwrap(), 1);
        assert_eq!(new_processor.records.len(), 1);
    }
}