
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

        let validated_data = self.validate_data(data)?;
        let normalized_data = self.normalize_data(&validated_data);
        let transformed_data = self.apply_transformations(&normalized_data);

        self.cache.insert(key.to_string(), transformed_data.clone());
        Ok(transformed_data)
    }

    fn validate_data(&self, data: &[f64]) -> Result<Vec<f64>, String> {
        let mut validated = Vec::with_capacity(data.len());
        
        for &value in data {
            if value.is_nan() || value.is_infinite() {
                return Err(format!("Invalid numeric value detected: {}", value));
            }
            validated.push(value);
        }
        
        Ok(validated)
    }

    fn normalize_data(&self, data: &[f64]) -> Vec<f64> {
        if data.len() < 2 {
            return data.to_vec();
        }

        let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if (max - min).abs() < f64::EPSILON {
            return vec![0.5; data.len()];
        }

        data.iter()
            .map(|&x| (x - min) / (max - min))
            .collect()
    }

    fn apply_transformations(&self, data: &[f64]) -> Vec<f64> {
        data.iter()
            .map(|&x| x.sqrt().abs())
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_stats(&self) -> (usize, usize) {
        let total_items: usize = self.cache.values().map(|v| v.len()).sum();
        (self.cache.len(), total_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let test_data = vec![1.0, 4.0, 9.0, 16.0];
        
        let result = processor.process_dataset("test", &test_data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), 4);
        
        let stats = processor.cache_stats();
        assert_eq!(stats.0, 1);
        assert_eq!(stats.1, 4);
    }

    #[test]
    fn test_invalid_data() {
        let mut processor = DataProcessor::new();
        let invalid_data = vec![1.0, f64::NAN, 3.0];
        
        let result = processor.process_dataset("invalid", &invalid_data);
        assert!(result.is_err());
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
    validation_rules: HashMap<String, ValidationRule>,
}

pub struct ValidationRule {
    min_value: Option<f64>,
    max_value: Option<f64>,
    required: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: HashMap::new(),
            validation_rules: HashMap::new(),
        }
    }

    pub fn add_dataset(&mut self, name: String, values: Vec<f64>) -> Result<(), String> {
        if self.data.contains_key(&name) {
            return Err(format!("Dataset '{}' already exists", name));
        }
        
        if let Some(rule) = self.validation_rules.get(&name) {
            if rule.required && values.is_empty() {
                return Err(format!("Dataset '{}' cannot be empty", name));
            }
            
            for &value in &values {
                if let Some(min) = rule.min_value {
                    if value < min {
                        return Err(format!("Value {} below minimum {}", value, min));
                    }
                }
                
                if let Some(max) = rule.max_value {
                    if value > max {
                        return Err(format!("Value {} above maximum {}", value, max));
                    }
                }
            }
        }
        
        self.data.insert(name, values);
        Ok(())
    }

    pub fn set_validation_rule(&mut self, dataset_name: String, rule: ValidationRule) {
        self.validation_rules.insert(dataset_name, rule);
    }

    pub fn calculate_statistics(&self, dataset_name: &str) -> Option<Statistics> {
        self.data.get(dataset_name).map(|values| {
            let count = values.len();
            let sum: f64 = values.iter().sum();
            let mean = if count > 0 { sum / count as f64 } else { 0.0 };
            
            let variance = if count > 1 {
                let squared_diff: f64 = values.iter()
                    .map(|&x| (x - mean).powi(2))
                    .sum();
                squared_diff / (count - 1) as f64
            } else {
                0.0
            };
            
            Statistics {
                count,
                sum,
                mean,
                variance,
                std_dev: variance.sqrt(),
            }
        })
    }

    pub fn transform_data<F>(&mut self, dataset_name: &str, transform_fn: F) -> Result<(), String>
    where
        F: Fn(f64) -> f64,
    {
        if let Some(values) = self.data.get_mut(dataset_name) {
            for value in values.iter_mut() {
                *value = transform_fn(*value);
            }
            Ok(())
        } else {
            Err(format!("Dataset '{}' not found", dataset_name))
        }
    }

    pub fn merge_datasets(&mut self, target_name: &str, source_name: &str) -> Result<(), String> {
        if target_name == source_name {
            return Err("Cannot merge dataset with itself".to_string());
        }

        let source_data = match self.data.remove(source_name) {
            Some(data) => data,
            None => return Err(format!("Source dataset '{}' not found", source_name)),
        };

        if let Some(target_data) = self.data.get_mut(target_name) {
            target_data.extend(source_data);
            Ok(())
        } else {
            self.data.insert(target_name.to_string(), source_data);
            Ok(())
        }
    }
}

pub struct Statistics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
}

impl ValidationRule {
    pub fn new() -> Self {
        ValidationRule {
            min_value: None,
            max_value: None,
            required: false,
        }
    }

    pub fn with_min(mut self, min: f64) -> Self {
        self.min_value = Some(min);
        self
    }

    pub fn with_max(mut self, max: f64) -> Self {
        self.max_value = Some(max);
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.add_dataset("test".to_string(), vec![1.0, 2.0, 3.0]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_duplicate_dataset() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("test".to_string(), vec![1.0]).unwrap();
        let result = processor.add_dataset("test".to_string(), vec![2.0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation() {
        let mut processor = DataProcessor::new();
        let rule = ValidationRule::new()
            .with_min(0.0)
            .with_max(10.0)
            .required();
        
        processor.set_validation_rule("test".to_string(), rule);
        
        let result = processor.add_dataset("test".to_string(), vec![5.0, 15.0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_statistics() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("values".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        
        let stats = processor.calculate_statistics("values").unwrap();
        assert_eq!(stats.count, 5);
        assert_eq!(stats.sum, 15.0);
        assert_eq!(stats.mean, 3.0);
    }
}