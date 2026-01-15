
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

    pub fn add_dataset(&mut self, key: String, values: Vec<f64>) -> Result<(), String> {
        if values.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        if values.iter().any(|&x| x.is_nan() || x.is_infinite()) {
            return Err("Dataset contains invalid numeric values".to_string());
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
        self.calculate_statistics(key).map(|stats| {
            self.data[key].iter()
                .map(|&x| (x - stats.min) / (stats.max - stats.min))
                .collect()
        })
    }

    pub fn get_keys(&self) -> Vec<String> {
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
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let result = processor.add_dataset(
            "temperatures".to_string(),
            vec![20.5, 22.1, 19.8, 23.4, 21.0]
        );
        
        assert!(result.is_ok());
        
        let stats = processor.calculate_statistics("temperatures").unwrap();
        assert_eq!(stats.count, 5);
        assert!((stats.mean - 21.36).abs() < 0.01);
        
        let normalized = processor.normalize_data("temperatures").unwrap();
        assert_eq!(normalized.len(), 5);
        assert!((normalized[0] - 0.194).abs() < 0.01);
    }

    #[test]
    fn test_invalid_data() {
        let mut processor = DataProcessor::new();
        
        let result = processor.add_dataset(
            "invalid".to_string(),
            vec![]
        );
        
        assert!(result.is_err());
        
        let result = processor.add_dataset(
            "nan_data".to_string(),
            vec![1.0, f64::NAN, 2.0]
        );
        
        assert!(result.is_err());
    }
}