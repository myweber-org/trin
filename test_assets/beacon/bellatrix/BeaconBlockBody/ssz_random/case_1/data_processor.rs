
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: HashMap::new(),
        }
    }

    pub fn add_dataset(&mut self, key: &str, values: Vec<f64>) -> Result<(), String> {
        if key.trim().is_empty() {
            return Err("Dataset key cannot be empty".to_string());
        }

        if values.is_empty() {
            return Err("Dataset values cannot be empty".to_string());
        }

        for &value in &values {
            if !value.is_finite() {
                return Err("Dataset contains invalid numeric values".to_string());
            }
        }

        self.data.insert(key.to_string(), values);
        Ok(())
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<DatasetStats> {
        self.data.get(key).map(|values| {
            let count = values.len();
            let sum: f64 = values.iter().sum();
            let mean = sum / count as f64;
            
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count as f64;
            
            let std_dev = variance.sqrt();

            DatasetStats {
                count,
                sum,
                mean,
                variance,
                std_dev,
            }
        })
    }

    pub fn normalize_data(&self, key: &str) -> Option<Vec<f64>> {
        self.data.get(key).map(|values| {
            let stats = self.calculate_statistics(key).unwrap();
            values.iter()
                .map(|&x| (x - stats.mean) / stats.std_dev)
                .collect()
        })
    }

    pub fn merge_datasets(&mut self, target_key: &str, source_key: &str) -> Result<(), String> {
        if !self.data.contains_key(target_key) || !self.data.contains_key(source_key) {
            return Err("One or both datasets do not exist".to_string());
        }

        let source_data = self.data.remove(source_key).unwrap();
        let target_data = self.data.get_mut(target_key).unwrap();
        
        target_data.extend(source_data);
        Ok(())
    }

    pub fn get_dataset_keys(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }
}

pub struct DatasetStats {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
}

impl std::fmt::Display for DatasetStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Count: {}, Sum: {:.2}, Mean: {:.2}, Variance: {:.2}, StdDev: {:.2}",
            self.count, self.sum, self.mean, self.variance, self.std_dev
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.add_dataset("temperatures", vec![20.5, 21.0, 22.3, 19.8]);
        assert!(result.is_ok());
        assert_eq!(processor.get_dataset_keys(), vec!["temperatures"]);
    }

    #[test]
    fn test_add_invalid_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.add_dataset("", vec![1.0, 2.0]);
        assert!(result.is_err());
        
        let result = processor.add_dataset("test", vec![]);
        assert!(result.is_err());
        
        let result = processor.add_dataset("test", vec![1.0, f64::NAN]);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_statistics() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("test", vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        
        let stats = processor.calculate_statistics("test").unwrap();
        assert_eq!(stats.count, 5);
        assert_eq!(stats.sum, 15.0);
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.variance, 2.0);
        assert_eq!(stats.std_dev, 2.0_f64.sqrt());
    }

    #[test]
    fn test_normalize_data() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("test", vec![1.0, 2.0, 3.0]).unwrap();
        
        let normalized = processor.normalize_data("test").unwrap();
        assert_eq!(normalized.len(), 3);
        
        let stats = processor.calculate_statistics("test").unwrap();
        let expected_mean = stats.mean;
        let expected_std = stats.std_dev;
        
        for (i, &value) in normalized.iter().enumerate() {
            let original = (i + 1) as f64;
            let expected = (original - expected_mean) / expected_std;
            assert!((value - expected).abs() < 1e-10);
        }
    }

    #[test]
    fn test_merge_datasets() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("set1", vec![1.0, 2.0]).unwrap();
        processor.add_dataset("set2", vec![3.0, 4.0]).unwrap();
        
        let result = processor.merge_datasets("set1", "set2");
        assert!(result.is_ok());
        
        let keys = processor.get_dataset_keys();
        assert_eq!(keys, vec!["set1"]);
        
        let stats = processor.calculate_statistics("set1").unwrap();
        assert_eq!(stats.count, 4);
        assert_eq!(stats.sum, 10.0);
    }
}