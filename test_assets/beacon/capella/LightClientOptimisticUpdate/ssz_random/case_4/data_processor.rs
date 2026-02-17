
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    values: Vec<f64>,
    metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    EmptyValues,
    ValueOutOfRange(f64),
    MissingMetadata(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::EmptyValues => write!(f, "Record contains no values"),
            DataError::ValueOutOfRange(val) => write!(f, "Value {} is out of acceptable range", val),
            DataError::MissingMetadata(key) => write!(f, "Missing required metadata: {}", key),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>, metadata: HashMap<String, String>) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        
        if values.is_empty() {
            return Err(DataError::EmptyValues);
        }
        
        for &value in &values {
            if !value.is_finite() {
                return Err(DataError::ValueOutOfRange(value));
            }
        }
        
        Ok(Self { id, values, metadata })
    }
    
    pub fn validate_metadata(&self, required_keys: &[&str]) -> Result<(), DataError> {
        for key in required_keys {
            if !self.metadata.contains_key(*key) {
                return Err(DataError::MissingMetadata(key.to_string()));
            }
        }
        Ok(())
    }
    
    pub fn transform_values<F>(&mut self, transform_fn: F) 
    where
        F: Fn(f64) -> f64,
    {
        self.values = self.values.iter().map(|&v| transform_fn(v)).collect();
    }
    
    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        let sum: f64 = self.values.iter().sum();
        let count = self.values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.values.iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
    
    pub fn get_id(&self) -> u32 {
        self.id
    }
    
    pub fn get_values(&self) -> &[f64] {
        &self.values
    }
    
    pub fn get_metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
}

pub fn process_records(records: &mut [DataRecord], required_metadata: &[&str]) -> Result<Vec<(u32, f64)>, DataError> {
    let mut results = Vec::new();
    
    for record in records {
        record.validate_metadata(required_metadata)?;
        
        record.transform_values(|v| v * 2.0);
        
        let (mean, _, _) = record.calculate_statistics();
        results.push((record.get_id(), mean));
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let metadata = HashMap::from([
            ("source".to_string(), "sensor_a".to_string()),
            ("timestamp".to_string(), "2024-01-15T10:30:00".to_string()),
        ]);
        
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0], metadata).unwrap();
        assert_eq!(record.get_id(), 1);
        assert_eq!(record.get_values(), &[1.0, 2.0, 3.0]);
    }
    
    #[test]
    fn test_invalid_id() {
        let result = DataRecord::new(0, vec![1.0], HashMap::new());
        assert!(matches!(result, Err(DataError::InvalidId)));
    }
    
    #[test]
    fn test_empty_values() {
        let result = DataRecord::new(1, vec![], HashMap::new());
        assert!(matches!(result, Err(DataError::EmptyValues)));
    }
    
    #[test]
    fn test_metadata_validation() {
        let metadata = HashMap::from([
            ("source".to_string(), "sensor_a".to_string()),
        ]);
        
        let record = DataRecord::new(1, vec![1.0, 2.0], metadata).unwrap();
        let result = record.validate_metadata(&["source", "timestamp"]);
        assert!(matches!(result, Err(DataError::MissingMetadata(_))));
    }
    
    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, vec![1.0, 2.0, 3.0], HashMap::new()).unwrap();
        record.transform_values(|v| v * 2.0);
        assert_eq!(record.get_values(), &[2.0, 4.0, 6.0]);
    }
    
    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0, 4.0], HashMap::new()).unwrap();
        let (mean, variance, std_dev) = record.calculate_statistics();
        assert_eq!(mean, 2.5);
        assert_eq!(variance, 1.25);
        assert_eq!(std_dev, 1.118033988749895);
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

    pub fn process_dataset(&mut self, dataset_name: &str, data: Vec<f64>) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        for rule in &self.validation_rules {
            if rule.required && data.iter().any(|&x| x.is_nan()) {
                return Err(format!("Field {} contains invalid values", rule.field_name));
            }

            if let Some(&value) = data.iter().find(|&&x| x < rule.min_value || x > rule.max_value) {
                return Err(format!("Value {} out of range for field {}", value, rule.field_name));
            }
        }

        let processed_data: Vec<f64> = data
            .iter()
            .map(|&x| x * 2.0)
            .filter(|&x| x > 0.0)
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
            let variance: f64 = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / count;
            
            Statistics {
                mean,
                variance,
                count: data.len(),
                min: *data.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0),
                max: *data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0),
            }
        })
    }
}

pub struct Statistics {
    pub mean: f64,
    pub variance: f64,
    pub count: usize,
    pub min: f64,
    pub max: f64,
}

impl ValidationRule {
    pub fn new(field_name: &str, min_value: f64, max_value: f64, required: bool) -> Self {
        ValidationRule {
            field_name: field_name.to_string(),
            min_value,
            max_value,
            required,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(ValidationRule::new("temperature", -50.0, 100.0, true));
        
        let data = vec![10.0, 20.0, 30.0, 40.0];
        let result = processor.process_dataset("test_data", data);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![20.0, 40.0, 60.0, 80.0]);
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(ValidationRule::new("pressure", 0.0, 100.0, true));
        
        let data = vec![50.0, 150.0, 75.0];
        let result = processor.process_dataset("invalid_data", data);
        
        assert!(result.is_err());
    }
}