
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
            return Err("Empty values vector".into());
        }
        Ok(())
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Vec<DataRecord> {
    records
        .into_iter()
        .filter(|record| record.validate().is_ok())
        .map(|mut record| {
            let normalized_values: Vec<f64> = record
                .values
                .iter()
                .map(|&value| {
                    if value < 0.0 {
                        0.0
                    } else if value > 100.0 {
                        100.0
                    } else {
                        value
                    }
                })
                .collect();
            record.values = normalized_values;
            record.add_metadata(
                "processed".to_string(),
                "true".to_string(),
            );
            record
        })
        .collect()
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if records.is_empty() {
        return stats;
    }

    let total_records = records.len() as f64;
    let mut value_sum = 0.0;
    let mut value_count = 0;

    for record in records {
        for &value in &record.values {
            value_sum += value;
            value_count += 1;
        }
    }

    if value_count > 0 {
        let average = value_sum / value_count as f64;
        stats.insert("average_value".to_string(), average);
        stats.insert("total_records".to_string(), total_records);
        stats.insert("total_values".to_string(), value_count as f64);
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 1234567890, vec![10.5, 20.3]);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, 1234567890, vec![10.5]);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord::new(1, 1000, vec![150.0, -5.0, 50.0]),
            DataRecord::new(2, 2000, vec![]),
        ];
        
        let processed = process_records(records);
        assert_eq!(processed.len(), 1);
        assert_eq!(processed[0].values, vec![100.0, 0.0, 50.0]);
        assert_eq!(processed[0].metadata.get("processed"), Some(&"true".to_string()));
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord::new(1, 1000, vec![10.0, 20.0]),
            DataRecord::new(2, 2000, vec![30.0, 40.0]),
        ];
        
        let stats = calculate_statistics(&records);
        assert_eq!(stats.get("average_value"), Some(&25.0));
        assert_eq!(stats.get("total_records"), Some(&2.0));
    }
}