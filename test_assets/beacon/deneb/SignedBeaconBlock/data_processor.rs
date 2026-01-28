
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>) -> Self {
        Self {
            id,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("Invalid ID: ID cannot be zero".to_string());
        }

        if self.values.is_empty() {
            return Err("Empty values array".to_string());
        }

        for (i, &value) in self.values.iter().enumerate() {
            if !value.is_finite() {
                return Err(format!("Invalid value at index {}: {}", i, value));
            }
        }

        Ok(())
    }

    pub fn transform(&mut self, factor: f64) {
        for value in &mut self.values {
            *value *= factor;
        }
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
}

pub fn process_records(records: &mut [DataRecord], factor: f64) -> Result<Vec<DataRecord>, String> {
    let mut processed = Vec::with_capacity(records.len());

    for record in records {
        record.validate()?;
        let mut processed_record = record.clone();
        processed_record.transform(factor);
        processed.push(processed_record);
    }

    Ok(processed)
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();

    if records.is_empty() {
        return stats;
    }

    let total_values: usize = records.iter().map(|r| r.values.len()).sum();
    let sum: f64 = records.iter().flat_map(|r| &r.values).sum();
    let count = total_values as f64;

    stats.insert("mean".to_string(), sum / count);
    stats.insert("total_records".to_string(), records.len() as f64);
    stats.insert("total_values".to_string(), count);

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, vec![1.0, 2.0]);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        record.transform(2.0);
        assert_eq!(record.values, vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_process_records() {
        let mut records = vec![
            DataRecord::new(1, vec![1.0, 2.0]),
            DataRecord::new(2, vec![3.0, 4.0]),
        ];

        let result = process_records(&mut records, 2.0);
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert_eq!(processed[0].values, vec![2.0, 4.0]);
        assert_eq!(processed[1].values, vec![6.0, 8.0]);
    }
}