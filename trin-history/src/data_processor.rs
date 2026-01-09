
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
        }
    }

    pub fn process_dataset(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty dataset provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let processed = Self::normalize_data(data)?;
        self.cache.insert(key.to_string(), processed.clone());
        
        Ok(processed)
    }

    fn normalize_data(data: &[f64]) -> Result<Vec<f64>, String> {
        let max_value = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if max_value <= 0.0 {
            return Err("Invalid data range for normalization".to_string());
        }

        Ok(data.iter().map(|&x| x / max_value).collect())
    }

    pub fn calculate_statistics(data: &[f64]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        let sum: f64 = data.iter().sum();
        let count = data.len() as f64;
        
        stats.insert("mean".to_string(), sum / count);
        stats.insert("sum".to_string(), sum);
        stats.insert("count".to_string(), count);
        
        if let Some(&min) = data.iter().min_by(|a, b| a.partial_cmp(b).unwrap()) {
            stats.insert("min".to_string(), min);
        }
        
        if let Some(&max) = data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()) {
            stats.insert("max".to_string(), max);
        }
        
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_data() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let result = DataProcessor::normalize_data(&data).unwrap();
        assert_eq!(result, vec![0.25, 0.5, 0.75, 1.0]);
    }

    #[test]
    fn test_empty_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.process_dataset("test", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_statistics() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let stats = DataProcessor::calculate_statistics(&data);
        
        assert_eq!(stats.get("mean"), Some(&2.5));
        assert_eq!(stats.get("sum"), Some(&10.0));
        assert_eq!(stats.get("count"), Some(&4.0));
        assert_eq!(stats.get("min"), Some(&1.0));
        assert_eq!(stats.get("max"), Some(&4.0));
    }
}