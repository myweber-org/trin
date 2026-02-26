
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

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.values.is_empty() {
            return None;
        }
        let sum: f64 = self.values.iter().sum();
        Some(sum / self.values.len() as f64)
    }

    pub fn normalize(&mut self) -> Result<(), &'static str> {
        let mean = match self.calculate_mean() {
            Some(m) => m,
            None => return Err("Cannot normalize empty data"),
        };

        let std_dev = self.calculate_std_dev().ok_or("Cannot compute standard deviation")?;
        
        if std_dev.abs() < f64::EPSILON {
            return Err("Standard deviation is zero, cannot normalize");
        }

        for value in &mut self.values {
            *value = (*value - mean) / std_dev;
        }

        Ok(())
    }

    fn calculate_std_dev(&self) -> Option<f64> {
        let mean = self.calculate_mean()?;
        let variance: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / self.values.len() as f64;
        
        Some(variance.sqrt())
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Vec<Result<(), String>> {
    records
        .iter_mut()
        .map(|record| {
            if !record.is_valid() {
                return Err(format!("Record {} is invalid", record.id));
            }
            
            match record.normalize() {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Failed to normalize record {}: {}", record.id, e)),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        assert!(record.is_valid());
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(0, vec![]);
        assert!(!record.is_valid());
    }

    #[test]
    fn test_mean_calculation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(record.calculate_mean(), Some(2.5));
    }

    #[test]
    fn test_normalization() {
        let mut record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        assert!(record.normalize().is_ok());
        
        let mean = record.calculate_mean().unwrap();
        assert!(mean.abs() < 1e-10);
    }

    #[test]
    fn test_metadata_operations() {
        let mut record = DataRecord::new(1, vec![1.0]);
        record.add_metadata("source".to_string(), "test".to_string());
        
        assert_eq!(record.get_metadata("source"), Some(&"test".to_string()));
        assert_eq!(record.get_metadata("nonexistent"), None);
    }
}