
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub min_value: f64,
    pub max_value: f64,
    pub required: bool,
}

impl DataProcessor {
    pub fn new(rules: Vec<ValidationRule>) -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: rules,
        }
    }

    pub fn process_dataset(&mut self, dataset_id: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty dataset provided".to_string());
        }

        self.validate_data(data)?;
        
        let processed_data = self.transform_data(data);
        self.cache.insert(dataset_id.to_string(), processed_data.clone());
        
        Ok(processed_data)
    }

    fn validate_data(&self, data: &[f64]) -> Result<(), String> {
        for rule in &self.validation_rules {
            if rule.required && data.is_empty() {
                return Err("Data required but empty".to_string());
            }

            for &value in data {
                if value < rule.min_value || value > rule.max_value {
                    return Err(format!(
                        "Value {} outside allowed range [{}, {}]",
                        value, rule.min_value, rule.max_value
                    ));
                }
            }
        }
        Ok(())
    }

    fn transform_data(&self, data: &[f64]) -> Vec<f64> {
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        data.iter()
            .map(|&x| (x - mean).abs())
            .collect()
    }

    pub fn get_cached_result(&self, dataset_id: &str) -> Option<&Vec<f64>> {
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
        let rules = vec![ValidationRule {
            min_value: 0.0,
            max_value: 100.0,
            required: true,
        }];

        let mut processor = DataProcessor::new(rules);
        let test_data = vec![10.0, 20.0, 30.0, 40.0];
        
        let result = processor.process_dataset("test1", &test_data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), test_data.len());
        
        let cached = processor.get_cached_result("test1");
        assert!(cached.is_some());
    }

    #[test]
    fn test_validation_failure() {
        let rules = vec![ValidationRule {
            min_value: 0.0,
            max_value: 50.0,
            required: true,
        }];

        let mut processor = DataProcessor::new(rules);
        let invalid_data = vec![10.0, 60.0, 30.0];
        
        let result = processor.process_dataset("test2", &invalid_data);
        assert!(result.is_err());
    }
}