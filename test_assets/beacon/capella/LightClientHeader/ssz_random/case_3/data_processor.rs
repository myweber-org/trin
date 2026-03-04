use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub field_name: String,
    pub min_value: f64,
    pub max_value: f64,
    pub required: bool,
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

    pub fn process_dataset(&mut self, dataset_name: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        for rule in &self.validation_rules {
            if rule.required && data.iter().any(|&x| x.is_nan()) {
                return Err(format!("Field '{}' contains invalid NaN values", rule.field_name));
            }

            if let Some(invalid_value) = data.iter().find(|&&x| x < rule.min_value || x > rule.max_value) {
                return Err(format!(
                    "Value {} in field '{}' outside allowed range [{}, {}]",
                    invalid_value, rule.field_name, rule.min_value, rule.max_value
                ));
            }
        }

        let processed_data: Vec<f64> = data
            .iter()
            .map(|&x| {
                if x.is_nan() {
                    0.0
                } else {
                    x * 2.0
                }
            })
            .collect();

        self.cache.insert(dataset_name.to_string(), processed_data.clone());

        Ok(processed_data)
    }

    pub fn get_cached_data(&self, dataset_name: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_name)
    }

    pub fn calculate_statistics(&self, dataset_name: &str) -> Option<Statistics> {
        self.cache.get(dataset_name).map(|data| {
            let sum: f64 = data.iter().sum();
            let count = data.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = data.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;
            
            Statistics {
                mean,
                variance,
                min: *data.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
                max: *data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
                count: data.len(),
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct Statistics {
    pub mean: f64,
    pub variance: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let rule = ValidationRule {
            field_name: "temperature".to_string(),
            min_value: -50.0,
            max_value: 100.0,
            required: true,
        };
        processor.add_validation_rule(rule);

        let test_data = vec![20.5, 25.0, 30.2, 18.7];
        let result = processor.process_dataset("weather", &test_data);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![41.0, 50.0, 60.4, 37.4]);
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        
        let rule = ValidationRule {
            field_name: "pressure".to_string(),
            min_value: 900.0,
            max_value: 1100.0,
            required: true,
        };
        processor.add_validation_rule(rule);

        let invalid_data = vec![850.0, 950.0, 1050.0];
        let result = processor.process_dataset("pressure_data", &invalid_data);
        
        assert!(result.is_err());
    }
}