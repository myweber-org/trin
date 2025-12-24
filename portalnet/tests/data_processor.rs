
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
                let validation_result = self.validate_dataset(values, rule);
                results.push(validation_result);
            } else if rule.required {
                results.push(ValidationResult {
                    field_name: rule.field_name.clone(),
                    is_valid: false,
                    message: "Required field not found".to_string(),
                });
            }
        }
        
        results
    }

    fn validate_dataset(&self, values: &[f64], rule: &ValidationRule) -> ValidationResult {
        let mut invalid_count = 0;
        
        for &value in values {
            if value < rule.min_value || value > rule.max_value {
                invalid_count += 1;
            }
        }

        let is_valid = invalid_count == 0;
        let message = if is_valid {
            "All values are within valid range".to_string()
        } else {
            format!("{} values out of range", invalid_count)
        };

        ValidationResult {
            field_name: rule.field_name.clone(),
            is_valid,
            message,
        }
    }

    pub fn calculate_statistics(&self, dataset_name: &str) -> Option<Statistics> {
        self.data.get(dataset_name).map(|values| {
            let count = values.len();
            let sum: f64 = values.iter().sum();
            let mean = sum / count as f64;
            
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count as f64;
            
            let std_dev = variance.sqrt();
            
            let mut sorted_values = values.clone();
            sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            let median = if count % 2 == 0 {
                (sorted_values[count / 2 - 1] + sorted_values[count / 2]) / 2.0
            } else {
                sorted_values[count / 2]
            };

            Statistics {
                count,
                mean,
                median,
                std_dev,
                min: *sorted_values.first().unwrap_or(&0.0),
                max: *sorted_values.last().unwrap_or(&0.0),
            }
        })
    }

    pub fn normalize_data(&mut self, dataset_name: &str) -> Result<(), String> {
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
    is_valid: bool,
    message: String,
}

pub struct Statistics {
    count: usize,
    mean: f64,
    median: f64,
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

impl Statistics {
    pub fn display(&self) -> String {
        format!(
            "Count: {}, Mean: {:.2}, Median: {:.2}, Std Dev: {:.2}, Range: [{:.2}, {:.2}]",
            self.count, self.mean, self.median, self.std_dev, self.min, self.max
        )
    }
}