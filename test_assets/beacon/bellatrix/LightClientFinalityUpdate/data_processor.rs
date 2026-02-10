
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
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    EmptyTags,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value cannot be negative"),
            ValidationError::EmptyTags => write!(f, "Record must have at least one tag"),
        }
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ValidationError> {
        Self::validate_record(&record)?;
        self.records.insert(record.id, record);
        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.values().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn filter_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|record| record.tags.iter().any(|t| t == tag))
            .collect()
    }

    pub fn transform_values<F>(&mut self, transform_fn: F) 
    where
        F: Fn(f64) -> f64,
    {
        for record in self.records.values_mut() {
            record.value = transform_fn(record.value);
        }
    }

    fn validate_record(record: &DataRecord) -> Result<(), ValidationError> {
        if record.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if record.name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        
        if record.value < 0.0 {
            return Err(ValidationError::NegativeValue);
        }
        
        if record.tags.is_empty() {
            return Err(ValidationError::EmptyTags);
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test Record".to_string(),
            value: 42.5,
            tags: vec!["test".to_string(), "sample".to_string()],
        };
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.records.len(), 1);
    }

    #[test]
    fn test_invalid_record_validation() {
        let record = DataRecord {
            id: 0,
            name: "".to_string(),
            value: -10.0,
            tags: vec![],
        };
        
        let mut processor = DataProcessor::new();
        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_calculate_average() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord {
                id: 1,
                name: "Record 1".to_string(),
                value: 10.0,
                tags: vec!["a".to_string()],
            },
            DataRecord {
                id: 2,
                name: "Record 2".to_string(),
                value: 20.0,
                tags: vec!["b".to_string()],
            },
            DataRecord {
                id: 3,
                name: "Record 3".to_string(),
                value: 30.0,
                tags: vec!["a".to_string(), "b".to_string()],
            },
        ];
        
        for record in records {
            processor.add_record(record).unwrap();
        }
        
        assert_eq!(processor.calculate_average(), 20.0);
    }

    #[test]
    fn test_filter_by_tag() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord {
                id: 1,
                name: "Record 1".to_string(),
                value: 10.0,
                tags: vec!["important".to_string()],
            },
            DataRecord {
                id: 2,
                name: "Record 2".to_string(),
                value: 20.0,
                tags: vec!["normal".to_string()],
            },
            DataRecord {
                id: 3,
                name: "Record 3".to_string(),
                value: 30.0,
                tags: vec!["important".to_string(), "urgent".to_string()],
            },
        ];
        
        for record in records {
            processor.add_record(record).unwrap();
        }
        
        let important_records = processor.filter_by_tag("important");
        assert_eq!(important_records.len(), 2);
    }

    #[test]
    fn test_transform_values() {
        let mut processor = DataProcessor::new();
        
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 10.0,
            tags: vec!["test".to_string()],
        };
        
        processor.add_record(record).unwrap();
        
        processor.transform_values(|x| x * 2.0);
        
        let updated_record = processor.get_record(1).unwrap();
        assert_eq!(updated_record.value, 20.0);
    }
}