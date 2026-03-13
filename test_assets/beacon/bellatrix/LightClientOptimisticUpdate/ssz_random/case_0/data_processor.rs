
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
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
    validation_rules: ValidationRules,
}

pub struct ValidationRules {
    min_value: f64,
    max_value: f64,
    required_keys: Vec<String>,
}

impl DataProcessor {
    pub fn new(rules: ValidationRules) -> Self {
        DataProcessor {
            data: HashMap::new(),
            validation_rules: rules,
        }
    }

    pub fn add_dataset(&mut self, key: String, values: Vec<f64>) -> Result<(), String> {
        if !self.validation_rules.required_keys.contains(&key) {
            return Err(format!("Key '{}' is not in required keys list", key));
        }

        for &value in &values {
            if value < self.validation_rules.min_value || value > self.validation_rules.max_value {
                return Err(format!("Value {} is outside allowed range [{}, {}]", 
                    value, self.validation_rules.min_value, self.validation_rules.max_value));
            }
        }

        self.data.insert(key, values);
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

            Statistics {
                count,
                sum,
                mean,
                variance,
                std_dev,
            }
        })
    }

    pub fn normalize_data(&mut self, key: &str) -> Result<(), String> {
        if let Some(values) = self.data.get_mut(key) {
            let stats = self.calculate_statistics(key).unwrap();
            
            for value in values {
                *value = (*value - stats.mean) / stats.std_dev;
            }
            Ok(())
        } else {
            Err(format!("Key '{}' not found in dataset", key))
        }
    }

    pub fn merge_datasets(&mut self, other: DataProcessor) {
        for (key, values) in other.data {
            self.data.entry(key)
                .and_modify(|existing| existing.extend_from_slice(&values))
                .or_insert(values);
        }
    }
}

pub struct Statistics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
}

impl ValidationRules {
    pub fn new(min_value: f64, max_value: f64, required_keys: Vec<String>) -> Self {
        ValidationRules {
            min_value,
            max_value,
            required_keys,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_validation() {
        let rules = ValidationRules::new(
            0.0,
            100.0,
            vec!["temperature".to_string(), "humidity".to_string()]
        );
        let mut processor = DataProcessor::new(rules);

        assert!(processor.add_dataset("temperature".to_string(), vec![25.5, 30.0, 22.8]).is_ok());
        assert!(processor.add_dataset("pressure".to_string(), vec![1013.25]).is_err());
        assert!(processor.add_dataset("temperature".to_string(), vec![-5.0]).is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let rules = ValidationRules::new(f64::MIN, f64::MAX, vec!["test".to_string()]);
        let mut processor = DataProcessor::new(rules);
        
        processor.add_dataset("test".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        let stats = processor.calculate_statistics("test").unwrap();
        
        assert_eq!(stats.count, 5);
        assert_eq!(stats.sum, 15.0);
        assert_eq!(stats.mean, 3.0);
    }
}