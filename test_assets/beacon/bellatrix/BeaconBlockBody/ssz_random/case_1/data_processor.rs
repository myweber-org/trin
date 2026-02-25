
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_value(&mut self, value: f64) -> &mut Self {
        self.values.push(value);
        self
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) -> &mut Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.id == 0 {
            return Err("Invalid record ID");
        }
        if self.timestamp < 0 {
            return Err("Timestamp cannot be negative");
        }
        if self.values.is_empty() {
            return Err("Record must contain at least one value");
        }
        Ok(())
    }

    pub fn calculate_statistics(&self) -> Option<DataStatistics> {
        if self.values.is_empty() {
            return None;
        }

        let count = self.values.len();
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count as f64;
        
        let variance: f64 = self.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count as f64;
        
        Some(DataStatistics {
            count,
            sum,
            mean,
            variance,
            min: *self.values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
            max: *self.values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct DataStatistics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub variance: f64,
    pub min: f64,
    pub max: f64,
}

pub fn process_records(records: &[DataRecord]) -> Vec<DataRecord> {
    records.iter()
        .filter(|record| record.validate().is_ok())
        .filter(|record| !record.values.iter().any(|&v| v.is_nan() || v.is_infinite()))
        .cloned()
        .collect()
}

pub fn transform_values<F>(records: &mut [DataRecord], transform_fn: F)
where
    F: Fn(f64) -> f64,
{
    for record in records.iter_mut() {
        for value in record.values.iter_mut() {
            *value = transform_fn(*value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1625097600);
        record.add_value(42.5);
        
        assert!(record.validate().is_ok());
        
        let invalid_record = DataRecord::new(0, 1625097600);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut record = DataRecord::new(1, 1625097600);
        record.add_value(10.0).add_value(20.0).add_value(30.0);
        
        let stats = record.calculate_statistics().unwrap();
        
        assert_eq!(stats.count, 3);
        assert_eq!(stats.sum, 60.0);
        assert_eq!(stats.mean, 20.0);
        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 30.0);
    }

    #[test]
    fn test_value_transformation() {
        let mut records = vec![
            {
                let mut r = DataRecord::new(1, 1625097600);
                r.add_value(1.0).add_value(2.0).add_value(3.0);
                r
            }
        ];
        
        transform_values(&mut records, |x| x * 2.0);
        
        assert_eq!(records[0].values, vec![2.0, 4.0, 6.0]);
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
    category: String,
}

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(Path::new(file_path))?;
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    fn save_filtered_to_csv(&self, category: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        
        let file = File::create(Path::new(output_path))?;
        let mut wtr = WriterBuilder::new().has_headers(true).from_writer(file);
        
        for record in filtered {
            wtr.serialize(record)?;
        }
        
        wtr.flush()?;
        Ok(())
    }

    fn add_record(&mut self, id: u32, name: String, value: f64, category: String) {
        self.records.push(Record {
            id,
            name,
            value,
            category,
        });
    }

    fn sort_by_value(&mut self) {
        self.records.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap());
    }
}

fn process_data_sample() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.add_record(1, "Item A".to_string(), 42.5, "Alpha".to_string());
    processor.add_record(2, "Item B".to_string(), 33.2, "Beta".to_string());
    processor.add_record(3, "Item C".to_string(), 55.7, "Alpha".to_string());
    processor.add_record(4, "Item D".to_string(), 27.9, "Gamma".to_string());
    
    println!("Total records: {}", processor.records.len());
    println!("Average value: {:.2}", processor.calculate_average());
    
    let alpha_records = processor.filter_by_category("Alpha");
    println!("Alpha category records: {}", alpha_records.len());
    
    processor.sort_by_value();
    println!("Sorted records:");
    for record in &processor.records {
        println!("  {}: {} = {}", record.id, record.name, record.value);
    }
    
    Ok(())
}
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u64, values: Vec<f64>) -> Self {
        Self {
            id,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.values.is_empty() && self.id > 0
    }

    pub fn calculate_statistics(&self) -> Option<DataStatistics> {
        if self.values.is_empty() {
            return None;
        }

        let sum: f64 = self.values.iter().sum();
        let count = self.values.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        Some(DataStatistics {
            mean,
            variance,
            count: self.values.len(),
            min: *self.values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
            max: *self.values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
        })
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn transform_values<F>(&mut self, transformer: F)
    where
        F: Fn(f64) -> f64,
    {
        self.values = self.values.iter().map(|&x| transformer(x)).collect();
    }
}

#[derive(Debug, Clone)]
pub struct DataStatistics {
    pub mean: f64,
    pub variance: f64,
    pub count: usize,
    pub min: f64,
    pub max: f64,
}

pub fn process_records(records: &[DataRecord]) -> Vec<DataRecord> {
    records
        .iter()
        .filter(|record| record.is_valid())
        .cloned()
        .collect()
}

pub fn normalize_values(records: &mut [DataRecord]) {
    for record in records {
        if let Some(stats) = record.calculate_statistics() {
            if stats.variance > 0.0 {
                record.transform_values(|x| (x - stats.mean) / stats.variance.sqrt());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(0, vec![]);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let stats = record.calculate_statistics().unwrap();
        
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.count, 5);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
    }

    #[test]
    fn test_metadata_operations() {
        let mut record = DataRecord::new(1, vec![1.0, 2.0]);
        record.add_metadata("source".to_string(), "sensor_a".to_string());
        
        assert_eq!(record.metadata.get("source"), Some(&"sensor_a".to_string()));
    }
}
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, timestamp: i64, values: Vec<f64>) -> Self {
        Self {
            id,
            timestamp,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.id == 0 {
            return Err("Invalid record ID".into());
        }
        if self.timestamp < 0 {
            return Err("Invalid timestamp".into());
        }
        if self.values.is_empty() {
            return Err("Empty values array".into());
        }
        Ok(())
    }
}

pub fn process_records(records: &[DataRecord]) -> Vec<DataRecord> {
    records
        .iter()
        .filter(|record| record.validate().is_ok())
        .map(|record| {
            let mut processed = record.clone();
            processed.values = record
                .values
                .iter()
                .map(|&value| value * 2.0)
                .collect();
            processed
        })
        .collect()
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    let total_records = records.len() as f64;

    if total_records == 0.0 {
        return stats;
    }

    let sum_values: f64 = records
        .iter()
        .flat_map(|record| record.values.iter())
        .sum();

    let avg_values = sum_values / (records.iter().map(|r| r.values.len()).sum::<usize>() as f64);

    stats.insert("total_records".to_string(), total_records);
    stats.insert("average_value".to_string(), avg_values);
    stats.insert("sum_values".to_string(), sum_values);

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 1234567890, vec![1.0, 2.0, 3.0]);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, 1234567890, vec![1.0]);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord::new(1, 1000, vec![1.0, 2.0]),
            DataRecord::new(2, 2000, vec![3.0, 4.0]),
        ];

        let processed = process_records(&records);
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].values, vec![2.0, 4.0]);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord::new(1, 1000, vec![1.0, 2.0]),
            DataRecord::new(2, 2000, vec![3.0, 4.0]),
        ];

        let stats = calculate_statistics(&records);
        assert_eq!(stats["total_records"], 2.0);
        assert_eq!(stats["average_value"], 2.5);
    }
}