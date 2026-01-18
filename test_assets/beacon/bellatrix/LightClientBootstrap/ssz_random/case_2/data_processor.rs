
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub min_value: f64,
    pub max_value: f64,
    pub required_fields: Vec<String>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: vec![
                ValidationRule {
                    min_value: 0.0,
                    max_value: 100.0,
                    required_fields: vec!["temperature".to_string(), "pressure".to_string()],
                },
            ],
        }
    }

    pub fn process_dataset(&mut self, dataset_id: &str, data: Vec<f64>) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        for rule in &self.validation_rules {
            if !self.validate_data(&data, rule) {
                return Err(format!("Data validation failed for rule: {:?}", rule));
            }
        }

        let processed_data = self.transform_data(data);
        self.cache.insert(dataset_id.to_string(), processed_data.clone());

        Ok(processed_data)
    }

    fn validate_data(&self, data: &[f64], rule: &ValidationRule) -> bool {
        data.iter().all(|&value| value >= rule.min_value && value <= rule.max_value)
    }

    fn transform_data(&self, mut data: Vec<f64>) -> Vec<f64> {
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        
        for value in &mut data {
            *value = (*value - mean).abs();
        }

        data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        data
    }

    pub fn get_cached_data(&self, dataset_id: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_id)
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
        let test_data = vec![10.5, 20.3, 30.7, 40.1];
        
        let result = processor.process_dataset("test_dataset", test_data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), 4);
        assert!(processed.windows(2).all(|w| w[0] <= w[1]));
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        let invalid_data = vec![-5.0, 150.0];
        
        let result = processor.process_dataset("invalid_dataset", invalid_data);
        assert!(result.is_err());
    }
}