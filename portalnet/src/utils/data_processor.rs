
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Self {
        DataRecord {
            id,
            value,
            category: category.to_string(),
            metadata: HashMap::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.category.is_empty() && self.value.is_finite()
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Vec<DataRecord> {
    records
        .iter_mut()
        .filter(|record| record.is_valid())
        .map(|record| {
            record.value *= 1.1;
            DataRecord {
                id: record.id,
                value: record.value,
                category: record.category.clone(),
                metadata: record.metadata.clone(),
            }
        })
        .collect()
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    if count == 0.0 {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (sum, mean, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 42.5, "category_a");
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, f64::NAN, "");
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_process_records() {
        let mut records = vec![
            DataRecord::new(1, 100.0, "test"),
            DataRecord::new(2, 200.0, ""),
        ];
        
        let processed = process_records(&mut records);
        assert_eq!(processed.len(), 1);
        assert_eq!(processed[0].value, 110.0);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            DataRecord::new(1, 10.0, "a"),
            DataRecord::new(2, 20.0, "b"),
            DataRecord::new(3, 30.0, "c"),
        ];
        
        let (sum, mean, std_dev) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(mean, 20.0);
        assert!((std_dev - 8.164965).abs() < 0.0001);
    }
}