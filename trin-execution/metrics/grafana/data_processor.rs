
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

    pub fn process_dataset(&mut self, dataset: &[HashMap<String, f64>]) -> Result<Vec<HashMap<String, f64>>, String> {
        let mut processed = Vec::new();

        for (index, record) in dataset.iter().enumerate() {
            match self.validate_record(record) {
                Ok(validated_record) => {
                    let transformed = self.transform_record(&validated_record);
                    processed.push(transformed);
                    self.cache_record(index, &transformed);
                }
                Err(e) => return Err(format!("Validation failed at record {}: {}", index, e)),
            }
        }

        Ok(processed)
    }

    fn validate_record(&self, record: &HashMap<String, f64>) -> Result<HashMap<String, f64>, String> {
        let mut validated = HashMap::new();

        for rule in &self.validation_rules {
            match record.get(&rule.field_name) {
                Some(&value) => {
                    if value < rule.min_value || value > rule.max_value {
                        return Err(format!(
                            "Field '{}' value {} out of range [{}, {}]",
                            rule.field_name, value, rule.min_value, rule.max_value
                        ));
                    }
                    validated.insert(rule.field_name.clone(), value);
                }
                None => {
                    if rule.required {
                        return Err(format!("Required field '{}' missing", rule.field_name));
                    }
                }
            }
        }

        Ok(validated)
    }

    fn transform_record(&self, record: &HashMap<String, f64>) -> HashMap<String, f64> {
        let mut transformed = HashMap::new();

        for (key, value) in record {
            let transformed_value = match key.as_str() {
                "temperature" => (value - 32.0) * 5.0 / 9.0,
                "pressure" => value * 1000.0,
                "humidity" => value.min(100.0).max(0.0),
                _ => *value,
            };
            transformed.insert(key.clone(), transformed_value);
        }

        transformed
    }

    fn cache_record(&mut self, index: usize, record: &HashMap<String, f64>) {
        for (key, value) in record {
            self.cache
                .entry(key.clone())
                .or_insert_with(Vec::new)
                .push(*value);
        }
    }

    pub fn get_cached_stats(&self, field: &str) -> Option<(f64, f64, f64)> {
        self.cache.get(field).map(|values| {
            let count = values.len() as f64;
            let sum: f64 = values.iter().sum();
            let mean = sum / count;
            let variance: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / count;
            let std_dev = variance.sqrt();

            (mean, variance, std_dev)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(ValidationRule {
            field_name: "temperature".to_string(),
            min_value: -50.0,
            max_value: 150.0,
            required: true,
        });

        let dataset = vec![
            [("temperature".to_string(), 68.0)].iter().cloned().collect(),
            [("temperature".to_string(), 32.0)].iter().cloned().collect(),
        ];

        let result = processor.process_dataset(&dataset);
        assert!(result.is_ok());
        
        if let Ok(processed) = result {
            assert_eq!(processed.len(), 2);
            assert!((processed[0]["temperature"] - 20.0).abs() < 0.001);
        }
    }
}