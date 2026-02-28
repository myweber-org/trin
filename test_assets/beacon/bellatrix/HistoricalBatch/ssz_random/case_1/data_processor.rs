
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
                return Err(format!("Value {} is out of allowed range [{}, {}]", 
                    value, self.validation_rules.min_value, self.validation_rules.max_value));
            }
        }

        self.data.insert(key, values);
        Ok(())
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<Statistics> {
        self.data.get(key).map(|values| {
            let sum: f64 = values.iter().sum();
            let count = values.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;
            
            let std_dev = variance.sqrt();
            
            let sorted_values = {
                let mut sorted = values.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                sorted
            };
            
            let median = if count as usize % 2 == 0 {
                let mid = count as usize / 2;
                (sorted_values[mid - 1] + sorted_values[mid]) / 2.0
            } else {
                sorted_values[count as usize / 2]
            };

            Statistics {
                mean,
                median,
                std_dev,
                min: *sorted_values.first().unwrap_or(&0.0),
                max: *sorted_values.last().unwrap_or(&0.0),
                count: values.len(),
            }
        })
    }

    pub fn normalize_data(&mut self, key: &str) -> Result<Vec<f64>, String> {
        if let Some(values) = self.data.get(key) {
            let stats = self.calculate_statistics(key).unwrap();
            
            if stats.std_dev == 0.0 {
                return Err("Cannot normalize data with zero standard deviation".to_string());
            }

            let normalized: Vec<f64> = values.iter()
                .map(|&x| (x - stats.mean) / stats.std_dev)
                .collect();

            self.data.insert(key.to_string(), normalized.clone());
            Ok(normalized)
        } else {
            Err(format!("Key '{}' not found in dataset", key))
        }
    }

    pub fn get_keys(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }

    pub fn clear_data(&mut self) {
        self.data.clear();
    }
}

pub struct Statistics {
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
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
    fn test_data_processor_creation() {
        let rules = ValidationRules::new(0.0, 100.0, vec!["temperature".to_string(), "humidity".to_string()]);
        let processor = DataProcessor::new(rules);
        assert_eq!(processor.get_keys().len(), 0);
    }

    #[test]
    fn test_add_valid_dataset() {
        let rules = ValidationRules::new(0.0, 100.0, vec!["temperature".to_string()]);
        let mut processor = DataProcessor::new(rules);
        
        let result = processor.add_dataset("temperature".to_string(), vec![20.5, 25.0, 30.2]);
        assert!(result.is_ok());
        assert_eq!(processor.get_keys(), vec!["temperature".to_string()]);
    }

    #[test]
    fn test_add_invalid_key() {
        let rules = ValidationRules::new(0.0, 100.0, vec!["temperature".to_string()]);
        let mut processor = DataProcessor::new(rules);
        
        let result = processor.add_dataset("pressure".to_string(), vec![20.5, 25.0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_statistics() {
        let rules = ValidationRules::new(0.0, 100.0, vec!["temperature".to_string()]);
        let mut processor = DataProcessor::new(rules);
        
        processor.add_dataset("temperature".to_string(), vec![10.0, 20.0, 30.0]).unwrap();
        let stats = processor.calculate_statistics("temperature").unwrap();
        
        assert_eq!(stats.mean, 20.0);
        assert_eq!(stats.median, 20.0);
        assert_eq!(stats.count, 3);
    }

    #[test]
    fn test_normalize_data() {
        let rules = ValidationRules::new(0.0, 100.0, vec!["temperature".to_string()]);
        let mut processor = DataProcessor::new(rules);
        
        processor.add_dataset("temperature".to_string(), vec![10.0, 20.0, 30.0]).unwrap();
        let result = processor.normalize_data("temperature");
        
        assert!(result.is_ok());
        let normalized = result.unwrap();
        assert_eq!(normalized.len(), 3);
    }
}