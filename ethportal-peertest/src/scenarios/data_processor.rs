
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

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("Invalid record ID".to_string());
        }
        
        if self.timestamp < 0 {
            return Err("Timestamp cannot be negative".to_string());
        }
        
        if self.values.is_empty() {
            return Err("Record must contain at least one value".to_string());
        }
        
        for (i, &value) in self.values.iter().enumerate() {
            if value.is_nan() || value.is_infinite() {
                return Err(format!("Invalid value at position {}", i));
            }
        }
        
        Ok(())
    }
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if records.is_empty() {
        return stats;
    }
    
    let all_values: Vec<f64> = records.iter()
        .flat_map(|r| r.values.iter())
        .copied()
        .collect();
    
    if !all_values.is_empty() {
        let sum: f64 = all_values.iter().sum();
        let count = all_values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = all_values.iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        let min = all_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = all_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("min".to_string(), min);
        stats.insert("max".to_string(), max);
        stats.insert("total_records".to_string(), records.len() as f64);
        stats.insert("total_values".to_string(), count);
    }
    
    stats
}

pub fn filter_records<F>(records: Vec<DataRecord>, predicate: F) -> Vec<DataRecord>
where
    F: Fn(&DataRecord) -> bool,
{
    records.into_iter().filter(predicate).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value(42.5);
        
        assert!(record.validate().is_ok());
        
        let invalid_record = DataRecord::new(0, 1234567890);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut record1 = DataRecord::new(1, 1000);
        record1.add_value(10.0).add_value(20.0);
        
        let mut record2 = DataRecord::new(2, 2000);
        record2.add_value(30.0).add_value(40.0);
        
        let records = vec![record1, record2];
        let stats = calculate_statistics(&records);
        
        assert_eq!(stats.get("mean"), Some(&25.0));
        assert_eq!(stats.get("total_records"), Some(&2.0));
    }

    #[test]
    fn test_record_filtering() {
        let records = vec![
            DataRecord::new(1, 1000),
            DataRecord::new(2, 2000),
            DataRecord::new(3, 3000),
        ];
        
        let filtered = filter_records(records, |r| r.id % 2 == 1);
        
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.id % 2 == 1));
    }
}