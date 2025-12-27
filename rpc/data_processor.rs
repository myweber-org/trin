
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
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
            data: HashMap::new(),
            validation_rules: Vec::new(),
        }
    }

    pub fn add_dataset(&mut self, name: String, values: Vec<f64>) -> Result<(), String> {
        if name.is_empty() {
            return Err("Dataset name cannot be empty".to_string());
        }

        if values.is_empty() {
            return Err("Dataset values cannot be empty".to_string());
        }

        self.data.insert(name, values);
        Ok(())
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn validate_all(&self) -> Vec<ValidationResult> {
        let mut results = Vec::new();

        for rule in &self.validation_rules {
            if let Some(values) = self.data.get(&rule.field_name) {
                let validation_result = self.validate_dataset(&rule, values);
                results.push(validation_result);
            } else if rule.required {
                results.push(ValidationResult {
                    field_name: rule.field_name.clone(),
                    valid: false,
                    message: "Required field not found".to_string(),
                });
            }
        }

        results
    }

    fn validate_dataset(&self, rule: &ValidationRule, values: &[f64]) -> ValidationResult {
        let mut invalid_count = 0;
        
        for &value in values {
            if value < rule.min_value || value > rule.max_value {
                invalid_count += 1;
            }
        }

        if invalid_count > 0 {
            ValidationResult {
                field_name: rule.field_name.clone(),
                valid: false,
                message: format!("{} values out of range", invalid_count),
            }
        } else {
            ValidationResult {
                field_name: rule.field_name.clone(),
                valid: true,
                message: "All values within valid range".to_string(),
            }
        }
    }

    pub fn calculate_statistics(&self, dataset_name: &str) -> Option<DatasetStatistics> {
        self.data.get(dataset_name).map(|values| {
            let count = values.len();
            let sum: f64 = values.iter().sum();
            let mean = sum / count as f64;
            
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count as f64;
            
            let std_dev = variance.sqrt();

            DatasetStatistics {
                count,
                mean,
                std_dev,
                min: *values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0),
                max: *values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0),
            }
        })
    }

    pub fn normalize_dataset(&mut self, dataset_name: &str) -> Result<(), String> {
        if let Some(values) = self.data.get_mut(dataset_name) {
            if values.is_empty() {
                return Err("Cannot normalize empty dataset".to_string());
            }

            let stats = self.calculate_statistics(dataset_name).unwrap();
            
            if stats.std_dev == 0.0 {
                return Err("Cannot normalize dataset with zero standard deviation".to_string());
            }

            for value in values.iter_mut() {
                *value = (*value - stats.mean) / stats.std_dev;
            }
            
            Ok(())
        } else {
            Err(format!("Dataset '{}' not found", dataset_name))
        }
    }
}

pub struct ValidationResult {
    field_name: String,
    valid: bool,
    message: String,
}

pub struct DatasetStatistics {
    count: usize,
    mean: f64,
    std_dev: f64,
    min: f64,
    max: f64,
}

impl ValidationRule {
    pub fn new(field_name: String, min_value: f64, max_value: f64, required: bool) -> Self {
        ValidationRule {
            field_name,
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
    fn test_add_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.add_dataset("test_data".to_string(), vec![1.0, 2.0, 3.0]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_calculate_statistics() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("test_data".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        
        let stats = processor.calculate_statistics("test_data").unwrap();
        assert_eq!(stats.count, 5);
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
    }

    #[test]
    fn test_validation() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("temperature".to_string(), vec![20.5, 25.0, 30.2, 15.8]).unwrap();
        
        let rule = ValidationRule::new("temperature".to_string(), 10.0, 35.0, true);
        processor.add_validation_rule(rule);
        
        let results = processor.validate_all();
        assert_eq!(results.len(), 1);
        assert!(results[0].valid);
    }
}