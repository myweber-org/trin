
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

    pub fn process_data(&mut self, dataset: &[HashMap<String, f64>]) -> Result<Vec<ProcessedRecord>, String> {
        let mut results = Vec::new();
        
        for (index, data) in dataset.iter().enumerate() {
            match self.validate_record(data) {
                Ok(_) => {
                    let processed = self.transform_record(data);
                    self.cache.insert(format!("record_{}", index), processed.values.clone());
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
                return Err(format!("Required field '{}' not found", rule.field_name));
            }
        }
        Ok(())
    }

    fn transform_record(&self, record: &HashMap<String, f64>) -> ProcessedRecord {
        let mut values = Vec::new();
        let mut stats = RecordStats::default();
        
        for (key, &value) in record {
            values.push(value);
            
            if value > stats.max_value {
                stats.max_value = value;
                stats.max_field = key.clone();
            }
            
            if value < stats.min_value {
                stats.min_value = value;
                stats.min_field = key.clone();
            }
            
            stats.sum += value;
            stats.count += 1;
        }
        
        if stats.count > 0 {
            stats.average = stats.sum / stats.count as f64;
        }
        
        ProcessedRecord {
            values,
            stats,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
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
    stats: RecordStats,
    timestamp: u64,
}

#[derive(Default)]
pub struct RecordStats {
    min_value: f64,
    max_value: f64,
    min_field: String,
    max_field: String,
    sum: f64,
    count: usize,
    average: f64,
}

impl ProcessedRecord {
    pub fn get_stats(&self) -> &RecordStats {
        &self.stats
    }
    
    pub fn get_values(&self) -> &[f64] {
        &self.values
    }
    
    pub fn get_timestamp(&self) -> u64 {
        self.timestamp
    }
}

impl RecordStats {
    pub fn display_summary(&self) -> String {
        format!(
            "Count: {}, Min: {} ({}), Max: {} ({}), Avg: {:.2}",
            self.count,
            self.min_value,
            self.min_field,
            self.max_value,
            self.max_field,
            self.average
        )
    }
}