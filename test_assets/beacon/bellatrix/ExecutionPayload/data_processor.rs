
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
                        return Err(format!("Field '{}' value {} out of range [{}, {}]", 
                            rule.field_name, value, rule.min_value, rule.max_value));
                    }
                    validated.insert(rule.field_name.clone(), value);
                }
                None => {
                    if rule.required {
                        return Err(format!("Required field '{}' not found", rule.field_name));
                    }
                }
            }
        }

        Ok(validated)
    }

    fn transform_record(&self, record: &HashMap<String, f64>) -> HashMap<String, f64> {
        let mut transformed = record.clone();
        
        for (key, value) in transformed.iter_mut() {
            if key.starts_with("normalized_") {
                *value = (*value * 100.0).round() / 100.0;
            }
        }

        transformed
    }

    fn cache_record(&mut self, index: usize, record: &HashMap<String, f64>) {
        let key = format!("record_{}", index);
        let values: Vec<f64> = record.values().copied().collect();
        self.cache.insert(key, values);
    }

    pub fn get_cached_stats(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        for (key, values) in &self.cache {
            if !values.is_empty() {
                let sum: f64 = values.iter().sum();
                let avg = sum / values.len() as f64;
                stats.insert(format!("avg_{}", key), avg);
            }
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        processor.add_validation_rule(ValidationRule {
            field_name: "temperature".to_string(),
            min_value: -50.0,
            max_value: 100.0,
            required: true,
        });

        let mut record = HashMap::new();
        record.insert("temperature".to_string(), 25.5);
        record.insert("normalized_pressure".to_string(), 0.876543);

        let dataset = vec![record];
        let result = processor.process_dataset(&dataset);
        
        assert!(result.is_ok());
        assert_eq!(processor.get_cached_stats().len(), 1);
    }
}