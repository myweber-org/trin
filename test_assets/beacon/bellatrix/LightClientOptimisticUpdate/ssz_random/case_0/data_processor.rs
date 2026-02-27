
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

    pub fn is_valid(&self) -> bool {
        !self.values.is_empty() && self.timestamp > 0
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.values.is_empty() {
            return None;
        }
        let sum: f64 = self.values.iter().sum();
        Some(sum / self.values.len() as f64)
    }
}

pub fn process_records(records: &[DataRecord]) -> Vec<DataRecord> {
    records
        .iter()
        .filter(|record| record.is_valid())
        .cloned()
        .collect()
}

pub fn transform_values(record: &mut DataRecord, transform_fn: fn(f64) -> f64) {
    for value in &mut record.values {
        *value = transform_fn(*value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square_transform(x: f64) -> f64 {
        x * x
    }

    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, 1234567890);
        assert_eq!(record.id, 1);
        assert_eq!(record.timestamp, 1234567890);
        assert!(record.values.is_empty());
    }

    #[test]
    fn test_record_validation() {
        let mut valid_record = DataRecord::new(2, 1234567890);
        valid_record.add_value(42.0);
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(3, 0);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_average_calculation() {
        let mut record = DataRecord::new(4, 1234567890);
        record.add_value(10.0).add_value(20.0).add_value(30.0);
        assert_eq!(record.calculate_average(), Some(20.0));
    }

    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord::new(5, 1234567890);
        record.add_value(2.0).add_value(3.0).add_value(4.0);
        transform_values(&mut record, square_transform);
        assert_eq!(record.values, vec![4.0, 9.0, 16.0]);
    }
}