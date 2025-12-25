
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
        let max_value = data
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .ok_or("Cannot find maximum value")?;

        if *max_value == 0.0 {
            return Err("Maximum value cannot be zero for normalization".to_string());
        }

        Ok(data.iter().map(|&x| x / max_value).collect())
    }

    pub fn calculate_statistics(data: &[f64]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if data.is_empty() {
            return stats;
        }

        let sum: f64 = data.iter().sum();
        let count = data.len() as f64;
        let mean = sum / count;

        let variance: f64 = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);

        stats
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_data() {
        let data = vec![2.0, 4.0, 6.0];
        let result = DataProcessor::normalize_data(&data).unwrap();
        assert_eq!(result, vec![1.0/3.0, 2.0/3.0, 1.0]);
    }

    #[test]
    fn test_calculate_statistics() {
        let data = vec![1.0, 2.0, 3.0];
        let stats = DataProcessor::calculate_statistics(&data);
        
        assert_eq!(stats.get("mean"), Some(&2.0));
        assert_eq!(stats.get("count"), Some(&3.0));
        assert_eq!(stats.get("sum"), Some(&6.0));
    }
}