
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
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_value(&mut self, value: f64) -> &mut Self {
        self.values.push(value);
        self
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) -> &mut Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.id == 0 {
            return Err("Invalid record ID");
        }
        if self.timestamp < 0 {
            return Err("Timestamp cannot be negative");
        }
        if self.values.is_empty() {
            return Err("Record must contain at least one value");
        }
        Ok(())
    }

    pub fn calculate_statistics(&self) -> Option<DataStatistics> {
        if self.values.is_empty() {
            return None;
        }

        let count = self.values.len();
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count as f64;
        let variance: f64 = self.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        Some(DataStatistics {
            count,
            sum,
            mean,
            variance,
            std_dev,
        })
    }
}

#[derive(Debug)]
pub struct DataStatistics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
}

pub fn process_records(records: &[DataRecord]) -> Vec<Result<DataStatistics, &'static str>> {
    records.iter()
        .map(|record| {
            record.validate()
                .and_then(|_| record.calculate_statistics()
                    .ok_or("Failed to calculate statistics"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let mut record = DataRecord::new(1, 1625097600);
        record.add_value(10.5)
              .add_value(20.3)
              .add_value(15.7)
              .add_metadata("source", "sensor_a");

        assert!(record.validate().is_ok());
        
        let stats = record.calculate_statistics().unwrap();
        assert_eq!(stats.count, 3);
        assert!((stats.mean - 15.5).abs() < 0.001);
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(0, 1625097600);
        assert_eq!(record.validate(), Err("Invalid record ID"));
    }

    #[test]
    fn test_process_records() {
        let mut valid_record = DataRecord::new(1, 1625097600);
        valid_record.add_value(5.0).add_value(10.0);

        let invalid_record = DataRecord::new(0, 1625097600);

        let records = vec![valid_record, invalid_record];
        let results = process_records(&records);

        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_err());
    }
}