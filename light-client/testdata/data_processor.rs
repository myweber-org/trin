use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<Box<dyn Fn(&[f64]) -> bool>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: vec![
                Box::new(|data| !data.is_empty()),
                Box::new(|data| data.iter().all(|&x| x.is_finite())),
                Box::new(|data| data.len() < 10000),
            ],
        }
    }

    pub fn process_dataset(&mut self, key: &str, data: Vec<f64>) -> Result<Vec<f64>, String> {
        if !self.validate_data(&data) {
            return Err("Data validation failed".to_string());
        }

        let processed = self.transform_data(data);
        self.cache.insert(key.to_string(), processed.clone());
        Ok(processed)
    }

    fn validate_data(&self, data: &[f64]) -> bool {
        self.validation_rules.iter().all(|rule| rule(data))
    }

    fn transform_data(&self, mut data: Vec<f64>) -> Vec<f64> {
        if data.len() < 2 {
            return data;
        }

        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let std_dev = (data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64).sqrt();

        if std_dev > 0.0 {
            for value in data.iter_mut() {
                *value = (*value - mean) / std_dev;
            }
        }

        data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        data
    }

    pub fn get_cached_result(&self, key: &str) -> Option<&Vec<f64>> {
        self.cache.get(key)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let result = processor.process_dataset("test", data).unwrap();
        assert_eq!(result.len(), 5);
        assert!(result[0] < result[4]);
    }

    #[test]
    fn test_invalid_data() {
        let mut processor = DataProcessor::new();
        let data = vec![];
        
        let result = processor.process_dataset("empty", data);
        assert!(result.is_err());
    }
}
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
                    self.cache_record(index, &processed.values);
                    results.push(processed);
                }
                Err(e) => return Err(format!("Validation failed at record {}: {}", index, e)),
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
                return Err(format!("Required field '{}' missing", rule.field_name));
            }
        }
        Ok(())
    }

    fn transform_record(&self, record: &HashMap<String, f64>) -> ProcessedRecord {
        let mut values = Vec::new();
        let mut stats = TransformationStats::new();

        for (key, &value) in record {
            let transformed = if key.contains("normalized") {
                (value - 50.0) / 100.0
            } else if key.contains("scaled") {
                value * 2.5
            } else {
                value
            };

            values.push(transformed);
            stats.update(transformed);
        }

        ProcessedRecord {
            values,
            stats,
            timestamp: std::time::SystemTime::now(),
        }
    }

    fn cache_record(&mut self, index: usize, values: &[f64]) {
        self.cache.insert(format!("record_{}", index), values.to_vec());
    }

    pub fn get_cached_data(&self, key: &str) -> Option<&Vec<f64>> {
        self.cache.get(key)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

pub struct ProcessedRecord {
    values: Vec<f64>,
    stats: TransformationStats,
    timestamp: std::time::SystemTime,
}

pub struct TransformationStats {
    count: usize,
    sum: f64,
    min: f64,
    max: f64,
}

impl TransformationStats {
    fn new() -> Self {
        TransformationStats {
            count: 0,
            sum: 0.0,
            min: f64::MAX,
            max: f64::MIN,
        }
    }

    fn update(&mut self, value: f64) {
        self.count += 1;
        self.sum += value;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }

    pub fn average(&self) -> f64 {
        if self.count > 0 {
            self.sum / self.count as f64
        } else {
            0.0
        }
    }

    pub fn range(&self) -> f64 {
        self.max - self.min
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
        record.insert("temperature".to_string(), 25.5);
        record.insert("humidity".to_string(), 65.0);

        let dataset = vec![record];
        let result = processor.process_dataset(&dataset);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(ValidationRule {
            field_name: "pressure".to_string(),
            min_value: 900.0,
            max_value: 1100.0,
            required: true,
        });

        let mut record = HashMap::new();
        record.insert("pressure".to_string(), 850.0);

        let dataset = vec![record];
        let result = processor.process_dataset(&dataset);
        
        assert!(result.is_err());
    }
}