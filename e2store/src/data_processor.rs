
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

    pub fn normalize_data(&self, dataset_name: &str) -> Option<Vec<f64>> {
        self.data.get(dataset_name).and_then(|values| {
            if values.is_empty() {
                return None;
            }
            
            let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            
            if (max - min).abs() < f64::EPSILON {
                return Some(vec![0.5; values.len()]);
            }
            
            let normalized: Vec<f64> = values.iter()
                .map(|&x| (x - min) / (max - min))
                .collect();
            
            Some(normalized)
        })
    }

    pub fn merge_datasets(&self, names: &[&str]) -> Option<Vec<f64>> {
        let mut result = Vec::new();
        
        for &name in names {
            if let Some(values) = self.data.get(name) {
                result.extend(values);
            } else {
                return None;
            }
        }
        
        Some(result)
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
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let rule = ValidationRule::new()
            .with_min(0.0)
            .with_max(100.0)
            .required();
        
        processor.set_validation_rule("scores".to_string(), rule);
        
        let result = processor.add_dataset(
            "scores".to_string(),
            vec![85.5, 92.0, 78.5, 95.0]
        );
        
        assert!(result.is_ok());
        
        let stats = processor.calculate_statistics("scores").unwrap();
        assert_eq!(stats.count, 4);
        assert!(stats.mean > 87.0);
        
        let normalized = processor.normalize_data("scores").unwrap();
        assert_eq!(normalized.len(), 4);
        assert!(normalized.iter().all(|&x| x >= 0.0 && x <= 1.0));
    }
}