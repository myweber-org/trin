
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

    pub fn add_dataset(&mut self, key: &str, values: Vec<f64>) {
        self.data.insert(key.to_string(), values);
    }

    pub fn set_validation_rule(&mut self, key: &str, rule: ValidationRule) {
        self.validation_rules.insert(key.to_string(), rule);
    }

    pub fn validate_dataset(&self, key: &str) -> Result<(), String> {
        let data = match self.data.get(key) {
            Some(d) => d,
            None => return Err(format!("Dataset '{}' not found", key)),
        };

        let rule = match self.validation_rules.get(key) {
            Some(r) => r,
            None => return Ok(()),
        };

        if rule.required && data.is_empty() {
            return Err(format!("Dataset '{}' is required but empty", key));
        }

        for &value in data {
            if let Some(min) = rule.min_value {
                if value < min {
                    return Err(format!("Value {} below minimum {} in dataset '{}'", value, min, key));
                }
            }
            
            if let Some(max) = rule.max_value {
                if value > max {
                    return Err(format!("Value {} above maximum {} in dataset '{}'", value, max, key));
                }
            }
        }

        Ok(())
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<Statistics> {
        let data = self.data.get(key)?;
        
        if data.is_empty() {
            return None;
        }

        let sum: f64 = data.iter().sum();
        let count = data.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        let mut sorted_data = data.clone();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let median = if count as usize % 2 == 0 {
            let mid = count as usize / 2;
            (sorted_data[mid - 1] + sorted_data[mid]) / 2.0
        } else {
            sorted_data[count as usize / 2]
        };

        Some(Statistics {
            mean,
            median,
            std_dev,
            min: *sorted_data.first().unwrap(),
            max: *sorted_data.last().unwrap(),
            count: data.len(),
        })
    }

    pub fn normalize_data(&self, key: &str) -> Option<Vec<f64>> {
        let data = self.data.get(key)?;
        
        if data.is_empty() {
            return None;
        }

        let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if (max - min).abs() < f64::EPSILON {
            return Some(vec![0.5; data.len()]);
        }

        Some(
            data.iter()
                .map(|&x| (x - min) / (max - min))
                .collect()
        )
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