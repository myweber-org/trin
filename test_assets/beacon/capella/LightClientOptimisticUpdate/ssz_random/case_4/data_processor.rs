
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: HashMap::new(),
        }
    }

    pub fn add_dataset(&mut self, key: &str, values: Vec<f64>) -> Result<(), String> {
        if values.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        if values.iter().any(|&x| x.is_nan() || x.is_infinite()) {
            return Err("Dataset contains invalid numeric values".to_string());
        }

        self.data.insert(key.to_string(), values);
        Ok(())
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<Statistics> {
        self.data.get(key).map(|values| {
            let count = values.len();
            let sum: f64 = values.iter().sum();
            let mean = sum / count as f64;
            
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count as f64;
            
            let std_dev = variance.sqrt();
            
            let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            
            Statistics {
                count,
                mean,
                std_dev,
                min,
                max,
            }
        })
    }

    pub fn normalize_data(&self, key: &str) -> Option<Vec<f64>> {
        self.data.get(key).map(|values| {
            let stats = self.calculate_statistics(key).unwrap();
            values.iter()
                .map(|&x| (x - stats.mean) / stats.std_dev)
                .collect()
        })
    }

    pub fn merge_datasets(&mut self, key1: &str, key2: &str, new_key: &str) -> Result<(), String> {
        let data1 = self.data.get(key1).cloned().ok_or("First dataset not found")?;
        let data2 = self.data.get(key2).cloned().ok_or("Second dataset not found")?;
        
        let mut merged = data1;
        merged.extend(data2);
        
        self.add_dataset(new_key, merged)
    }

    pub fn list_datasets(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }
}

pub struct Statistics {
    pub count: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_calculate() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        assert!(processor.add_dataset("test", data).is_ok());
        let stats = processor.calculate_statistics("test").unwrap();
        
        assert_eq!(stats.count, 5);
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
    }

    #[test]
    fn test_invalid_data() {
        let mut processor = DataProcessor::new();
        let invalid_data = vec![1.0, f64::NAN, 3.0];
        
        assert!(processor.add_dataset("invalid", invalid_data).is_err());
    }

    #[test]
    fn test_normalization() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        processor.add_dataset("norm", data).unwrap();
        let normalized = processor.normalize_data("norm").unwrap();
        
        let mean: f64 = normalized.iter().sum::<f64>() / normalized.len() as f64;
        let variance: f64 = normalized.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / normalized.len() as f64;
        
        assert!(mean.abs() < 1e-10);
        assert!((variance - 1.0).abs() < 1e-10);
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    value: f64,
    category: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn get_summary(&self) -> String {
        let avg = self.calculate_average().unwrap_or(0.0);
        let count = self.count_records();
        let max = self.find_max_value()
            .map(|r| r.value)
            .unwrap_or(0.0);
        
        format!("Records: {}, Average: {:.2}, Max: {:.2}", count, avg, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,value,category").unwrap();
        writeln!(file, "1,10.5,A").unwrap();
        writeln!(file, "2,20.3,B").unwrap();
        writeln!(file, "3,15.7,A").unwrap();
        writeln!(file, "4,25.1,B").unwrap();
        file
    }

    #[test]
    fn test_data_processing() {
        let test_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        processor.load_from_csv(test_file.path().to_str().unwrap())
            .expect("Failed to load CSV");
        
        assert_eq!(processor.count_records(), 4);
        assert_eq!(processor.calculate_average(), Some(17.9));
        
        let category_a = processor.filter_by_category("A");
        assert_eq!(category_a.len(), 2);
        
        let max_record = processor.find_max_value().unwrap();
        assert_eq!(max_record.value, 25.1);
        assert_eq!(max_record.category, "B");
    }
}
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
    pub timestamp: String,
}

impl DataRecord {
    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() &&
        self.value >= 0.0 &&
        !self.category.is_empty() &&
        !self.timestamp.is_empty()
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);

        let mut count = 0;
        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }
        Ok(count)
    }

    pub fn save_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        let mut wtr = WriterBuilder::new()
            .has_headers(true)
            .from_writer(file);

        for record in &self.records {
            wtr.serialize(record)?;
        }
        wtr.flush()?;
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
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

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), &'static str> {
        if record.is_valid() {
            self.records.push(record);
            Ok(())
        } else {
            Err("Invalid record data")
        }
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
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
        let valid_record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 10.5,
            category: "A".to_string(),
            timestamp: "2024-01-01".to_string(),
        };
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord {
            id: 2,
            name: "".to_string(),
            value: -5.0,
            category: "B".to_string(),
            timestamp: "2024-01-01".to_string(),
        };
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();
        
        let record = DataRecord {
            id: 1,
            name: "Item1".to_string(),
            value: 100.0,
            category: "Electronics".to_string(),
            timestamp: "2024-01-01T10:00:00".to_string(),
        };

        assert!(processor.add_record(record.clone()).is_ok());
        assert_eq!(processor.get_record_count(), 1);
        
        let filtered = processor.filter_by_category("Electronics");
        assert_eq!(filtered.len(), 1);
        
        let avg = processor.calculate_average_value();
        assert_eq!(avg, Some(100.0));
        
        processor.clear();
        assert_eq!(processor.get_record_count(), 0);
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let mut processor = DataProcessor::new();
        
        let temp_file = NamedTempFile::new()?;
        let test_records = vec![
            DataRecord {
                id: 1,
                name: "Test1".to_string(),
                value: 50.0,
                category: "A".to_string(),
                timestamp: "2024-01-01".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Test2".to_string(),
                value: 75.0,
                category: "B".to_string(),
                timestamp: "2024-01-02".to_string(),
            },
        ];

        for record in test_records {
            processor.add_record(record)?;
        }

        let save_path = temp_file.path();
        processor.save_to_csv(save_path)?;

        let mut new_processor = DataProcessor::new();
        let loaded_count = new_processor.load_from_csv(save_path)?;
        
        assert_eq!(loaded_count, 2);
        assert_eq!(new_processor.get_record_count(), 2);
        
        Ok(())
    }
}