
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

    pub fn process_dataset(&mut self, dataset: &[HashMap<String, f64>]) -> Result<Vec<ProcessedRecord>, String> {
        let mut results = Vec::new();
        
        for (index, record) in dataset.iter().enumerate() {
            match self.validate_record(record) {
                Ok(_) => {
                    let processed = self.transform_record(record);
                    self.cache_record(&processed);
                    results.push(processed);
                }
                Err(err) => {
                    return Err(format!("Validation failed at record {}: {}", index, err));
                }
            }
        }
        
        Ok(results)
    }

    fn validate_record(&self, record: &HashMap<String, f64>) -> Result<(), String> {
        for rule in &self.validation_rules {
            if let Some(&value) = record.get(&rule.field_name) {
                if value < rule.min_value || value > rule.max_value {
                    return Err(format!("Field '{}' value {} out of range [{}, {}]", 
                        rule.field_name, value, rule.min_value, rule.max_value));
                }
            } else if rule.required {
                return Err(format!("Required field '{}' not found", rule.field_name));
            }
        }
        Ok(())
    }

    fn transform_record(&self, record: &HashMap<String, f64>) -> ProcessedRecord {
        let mut normalized = HashMap::new();
        let mut statistics = RecordStats::default();
        
        for (key, &value) in record {
            let transformed = (value - 100.0) / 50.0;
            normalized.insert(key.clone(), transformed);
            
            statistics.update(value);
        }
        
        ProcessedRecord {
            original: record.clone(),
            normalized,
            statistics,
        }
    }

    fn cache_record(&mut self, record: &ProcessedRecord) {
        for (key, &value) in &record.normalized {
            self.cache
                .entry(key.clone())
                .or_insert_with(Vec::new)
                .push(value);
        }
    }

    pub fn get_cached_average(&self, field: &str) -> Option<f64> {
        self.cache.get(field).map(|values| {
            values.iter().sum::<f64>() / values.len() as f64
        })
    }
}

pub struct ProcessedRecord {
    original: HashMap<String, f64>,
    normalized: HashMap<String, f64>,
    statistics: RecordStats,
}

#[derive(Default)]
pub struct RecordStats {
    count: usize,
    sum: f64,
    min: f64,
    max: f64,
}

impl RecordStats {
    fn update(&mut self, value: f64) {
        self.count += 1;
        self.sum += value;
        
        if self.count == 1 {
            self.min = value;
            self.max = value;
        } else {
            self.min = self.min.min(value);
            self.max = self.max.max(value);
        }
    }

    pub fn average(&self) -> f64 {
        if self.count > 0 {
            self.sum / self.count as f64
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(ValidationRule {
            field_name: "temperature".to_string(),
            min_value: -50.0,
            max_value: 150.0,
            required: true,
        });

        let mut record = HashMap::new();
        record.insert("temperature".to_string(), 25.0);
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(ValidationRule {
            field_name: "pressure".to_string(),
            min_value: 0.0,
            max_value: 100.0,
            required: true,
        });

        let mut record = HashMap::new();
        record.insert("pressure".to_string(), 150.0);
        
        assert!(processor.validate_record(&record).is_err());
    }
}