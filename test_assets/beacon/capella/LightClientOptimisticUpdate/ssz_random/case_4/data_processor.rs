
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
        if values.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        if values.iter().any(|&x| x.is_nan() || x.is_infinite()) {
            return Err("Dataset contains invalid numeric values".to_string());
        }

        self.data.insert(key.to_string(), values);
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
            
            let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            
            Statistics {
                count,
                mean,
                std_dev,
                min,
                max,
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

    pub fn merge_datasets(&mut self, key1: &str, key2: &str, new_key: &str) -> Result<(), String> {
        let data1 = self.data.get(key1).cloned().ok_or("First dataset not found")?;
        let data2 = self.data.get(key2).cloned().ok_or("Second dataset not found")?;
        
        let mut merged = data1;
        merged.extend(data2);
        
        self.add_dataset(new_key, merged)
    }

    pub fn list_datasets(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }
}

pub struct Statistics {
    pub count: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_calculate() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        assert!(processor.add_dataset("test", data).is_ok());
        let stats = processor.calculate_statistics("test").unwrap();
        
        assert_eq!(stats.count, 5);
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
    }

    #[test]
    fn test_invalid_data() {
        let mut processor = DataProcessor::new();
        let invalid_data = vec![1.0, f64::NAN, 3.0];
        
        assert!(processor.add_dataset("invalid", invalid_data).is_err());
    }

    #[test]
    fn test_normalization() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        processor.add_dataset("norm", data).unwrap();
        let normalized = processor.normalize_data("norm").unwrap();
        
        let mean: f64 = normalized.iter().sum::<f64>() / normalized.len() as f64;
        let variance: f64 = normalized.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / normalized.len() as f64;
        
        assert!(mean.abs() < 1e-10);
        assert!((variance - 1.0).abs() < 1e-10);
    }
}