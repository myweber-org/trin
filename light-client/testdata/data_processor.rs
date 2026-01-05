use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<Box<dyn Fn(&[f64]) -> bool>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: vec![
                Box::new(|data| !data.is_empty()),
                Box::new(|data| data.iter().all(|&x| x.is_finite())),
                Box::new(|data| data.len() < 10000),
            ],
        }
    }

    pub fn process_dataset(&mut self, key: &str, data: Vec<f64>) -> Result<Vec<f64>, String> {
        if !self.validate_data(&data) {
            return Err("Data validation failed".to_string());
        }

        let processed = self.transform_data(data);
        self.cache.insert(key.to_string(), processed.clone());
        Ok(processed)
    }

    fn validate_data(&self, data: &[f64]) -> bool {
        self.validation_rules.iter().all(|rule| rule(data))
    }

    fn transform_data(&self, mut data: Vec<f64>) -> Vec<f64> {
        if data.len() < 2 {
            return data;
        }

        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let std_dev = (data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64).sqrt();

        if std_dev > 0.0 {
            for value in data.iter_mut() {
                *value = (*value - mean) / std_dev;
            }
        }

        data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        data
    }

    pub fn get_cached_result(&self, key: &str) -> Option<&Vec<f64>> {
        self.cache.get(key)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let result = processor.process_dataset("test", data).unwrap();
        assert_eq!(result.len(), 5);
        assert!(result[0] < result[4]);
    }

    #[test]
    fn test_invalid_data() {
        let mut processor = DataProcessor::new();
        let data = vec![];
        
        let result = processor.process_dataset("empty", data);
        assert!(result.is_err());
    }
}