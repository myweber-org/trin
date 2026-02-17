
use std::collections::HashMap;

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

    pub fn is_valid(&self) -> bool {
        !self.values.is_empty() && self.timestamp > 0
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
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
}

#[derive(Debug, Clone)]
pub struct DataStatistics {
    pub mean: f64,
    pub variance: f64,
    pub count: usize,
    pub min: f64,
    pub max: f64,
}

pub fn normalize_values(values: &[f64]) -> Vec<f64> {
    if values.is_empty() {
        return Vec::new();
    }

    let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let range = max - min;

    if range == 0.0 {
        return vec![0.5; values.len()];
    }

    values.iter()
        .map(|&x| (x - min) / range)
        .collect()
}

pub fn filter_records(records: &[DataRecord], predicate: impl Fn(&DataRecord) -> bool) -> Vec<DataRecord> {
    records.iter()
        .filter(|record| predicate(record))
        .cloned()
        .collect()
}

pub fn merge_records(records: &[DataRecord]) -> Option<DataRecord> {
    if records.is_empty() {
        return None;
    }

    let mut merged_values = Vec::new();
    let mut merged_metadata = HashMap::new();
    let mut timestamp_sum = 0;

    for record in records {
        merged_values.extend_from_slice(&record.values);
        timestamp_sum += record.timestamp;
        
        for (key, value) in &record.metadata {
            merged_metadata.insert(key.clone(), value.clone());
        }
    }

    let avg_timestamp = timestamp_sum / records.len() as i64;
    
    Some(DataRecord {
        id: records[0].id,
        timestamp: avg_timestamp,
        values: merged_values,
        metadata: merged_metadata,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 1234567890, vec![1.0, 2.0, 3.0]);
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, 0, vec![]);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_normalization() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let normalized = normalize_values(&values);
        
        assert_eq!(normalized.len(), 5);
        assert_eq!(normalized[0], 0.0);
        assert_eq!(normalized[4], 1.0);
    }

    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord::new(1, 1234567890, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let stats = record.calculate_statistics().unwrap();
        
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.count, 5);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
    }
}use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

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

    fn load_from_csv(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
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
        
        let file = File::create(output_path)?;
        let mut wtr = WriterBuilder::new().has_headers(true).from_writer(file);
        
        for record in filtered {
            wtr.serialize(record)?;
        }
        
        wtr.flush()?;
        Ok(())
    }

    fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

fn process_data_sample() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.load_from_csv("input_data.csv")?;
    
    println!("Total records loaded: {}", processor.records.len());
    println!("Average value: {:.2}", processor.calculate_average());
    
    if let Some(max_record) = processor.find_max_value() {
        println!("Max value record: {:?}", max_record);
    }
    
    let filtered = processor.filter_by_category("premium");
    println!("Premium records found: {}", filtered.len());
    
    processor.save_filtered_to_csv("premium", "premium_records.csv")?;
    
    Ok(())
}