
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub fn add_metadata(&mut self, key: String, value: String) -> &mut Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("ID cannot be zero".to_string());
        }
        if self.timestamp < 0 {
            return Err("Timestamp cannot be negative".to_string());
        }
        if self.values.is_empty() {
            return Err("Values cannot be empty".to_string());
        }
        Ok(())
    }

    pub fn calculate_statistics(&self) -> Statistics {
        let count = self.values.len();
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count as f64;
        let variance: f64 = self.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        Statistics {
            count,
            sum,
            mean,
            variance,
            std_dev,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
}

pub fn process_records(records: &[DataRecord]) -> Vec<Result<Statistics, String>> {
    records.iter()
        .map(|record| {
            record.validate()?;
            Ok(record.calculate_statistics())
        })
        .collect()
}

pub fn filter_valid_records(records: Vec<DataRecord>) -> (Vec<DataRecord>, Vec<DataRecord>) {
    let mut valid = Vec::new();
    let mut invalid = Vec::new();

    for record in records {
        match record.validate() {
            Ok(_) => valid.push(record),
            Err(_) => invalid.push(record),
        }
    }

    (valid, invalid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value(10.5);
        
        assert!(record.validate().is_ok());
        
        let invalid_record = DataRecord::new(0, 1234567890);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value(10.0).add_value(20.0).add_value(30.0);
        
        let stats = record.calculate_statistics();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.sum, 60.0);
        assert_eq!(stats.mean, 20.0);
    }
}