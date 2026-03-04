
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

pub struct ValidationRule {
    field_name: String,
    min_value: f64,
    max_value: f64,
    required: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: Vec::new(),
        }
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn process_dataset(&mut self, dataset_name: &str, data: Vec<f64>) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        for rule in &self.validation_rules {
            if !self.validate_data(&data, rule) {
                return Err(format!("Validation failed for rule: {}", rule.field_name));
            }
        }

        let processed_data = self.transform_data(data);
        self.cache.insert(dataset_name.to_string(), processed_data.clone());
        
        Ok(processed_data)
    }

    pub fn get_cached_data(&self, dataset_name: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_name)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    fn validate_data(&self, data: &Vec<f64>, rule: &ValidationRule) -> bool {
        if rule.required && data.is_empty() {
            return false;
        }

        for &value in data {
            if value < rule.min_value || value > rule.max_value {
                return false;
            }
        }
        true
    }

    fn transform_data(&self, data: Vec<f64>) -> Vec<f64> {
        let mean = self.calculate_mean(&data);
        data.into_iter()
            .map(|x| (x - mean).abs())
            .collect()
    }

    fn calculate_mean(&self, data: &Vec<f64>) -> f64 {
        let sum: f64 = data.iter().sum();
        sum / data.len() as f64
    }
}

impl ValidationRule {
    pub fn new(field_name: &str, min_value: f64, max_value: f64, required: bool) -> Self {
        ValidationRule {
            field_name: field_name.to_string(),
            min_value,
            max_value,
            required,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let rule = ValidationRule::new("temperature", -50.0, 150.0, true);
        processor.add_validation_rule(rule);

        let test_data = vec![25.5, 30.2, 22.8, 18.9];
        let result = processor.process_dataset("weather", test_data);
        
        assert!(result.is_ok());
        assert_eq!(processor.get_cached_data("weather").unwrap().len(), 4);
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        let rule = ValidationRule::new("pressure", 0.0, 100.0, true);
        processor.add_validation_rule(rule);

        let invalid_data = vec![25.0, 30.0, 150.0];
        let result = processor.process_dataset("invalid", invalid_data);
        
        assert!(result.is_err());
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ValidationError {
    message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error: {}", self.message)
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Result<Self, ValidationError> {
        if threshold < 0.0 || threshold > 1.0 {
            return Err(ValidationError {
                message: format!("Threshold {} must be between 0.0 and 1.0", threshold),
            });
        }
        Ok(Self { threshold })
    }

    pub fn process_data(&self, data: &[f64]) -> Result<Vec<f64>, ValidationError> {
        if data.is_empty() {
            return Err(ValidationError {
                message: "Input data cannot be empty".to_string(),
            });
        }

        let filtered_data: Vec<f64> = data
            .iter()
            .filter(|&&value| value >= self.threshold)
            .cloned()
            .collect();

        if filtered_data.is_empty() {
            return Err(ValidationError {
                message: format!(
                    "No data points above threshold {} found",
                    self.threshold
                ),
            });
        }

        let mean = filtered_data.iter().sum::<f64>() / filtered_data.len() as f64;
        let normalized_data: Vec<f64> = filtered_data
            .iter()
            .map(|&value| (value - mean).abs())
            .collect();

        Ok(normalized_data)
    }

    pub fn calculate_statistics(&self, data: &[f64]) -> Result<(f64, f64, f64), ValidationError> {
        if data.len() < 2 {
            return Err(ValidationError {
                message: "At least two data points required for statistics".to_string(),
            });
        }

        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = data
            .iter()
            .map(|&value| (value - mean).powi(2))
            .sum::<f64>()
            / (data.len() - 1) as f64;
        let std_dev = variance.sqrt();

        Ok((mean, variance, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_processor_creation() {
        let processor = DataProcessor::new(0.5);
        assert!(processor.is_ok());
    }

    #[test]
    fn test_invalid_processor_creation() {
        let processor = DataProcessor::new(1.5);
        assert!(processor.is_err());
    }

    #[test]
    fn test_process_data() {
        let processor = DataProcessor::new(0.3).unwrap();
        let data = vec![0.1, 0.4, 0.5, 0.2, 0.8];
        let result = processor.process_data(&data);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.len(), 3);
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(0.0).unwrap();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = processor.calculate_statistics(&data);
        assert!(stats.is_ok());
        let (mean, variance, std_dev) = stats.unwrap();
        assert_eq!(mean, 3.0);
        assert!((variance - 2.5).abs() < 1e-10);
        assert!((std_dev - 1.5811388300841898).abs() < 1e-10);
    }
}