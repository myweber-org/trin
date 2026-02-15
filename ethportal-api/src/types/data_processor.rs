
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum DataError {
    InvalidFormat,
    OutOfRange,
    EmptyData,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidFormat => write!(f, "Data format is invalid"),
            DataError::OutOfRange => write!(f, "Value is out of acceptable range"),
            DataError::EmptyData => write!(f, "No data provided"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Result<Self, DataError> {
        if threshold <= 0.0 || threshold > 100.0 {
            return Err(DataError::OutOfRange);
        }
        Ok(Self { threshold })
    }

    pub fn process_values(&self, values: &[f64]) -> Result<Vec<f64>, DataError> {
        if values.is_empty() {
            return Err(DataError::EmptyData);
        }

        let mut result = Vec::with_capacity(values.len());
        for &value in values {
            if value.is_nan() || value.is_infinite() {
                return Err(DataError::InvalidFormat);
            }
            let processed = if value > self.threshold {
                value * 0.9
            } else {
                value * 1.1
            };
            result.push(processed);
        }
        Ok(result)
    }

    pub fn calculate_statistics(&self, values: &[f64]) -> Result<(f64, f64, f64), DataError> {
        if values.is_empty() {
            return Err(DataError::EmptyData);
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;
        
        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        
        let std_dev = variance.sqrt();
        
        Ok((mean, variance, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_processor_creation() {
        let processor = DataProcessor::new(50.0);
        assert!(processor.is_ok());
    }

    #[test]
    fn test_invalid_threshold() {
        let processor = DataProcessor::new(0.0);
        assert!(processor.is_err());
        
        let processor = DataProcessor::new(150.0);
        assert!(processor.is_err());
    }

    #[test]
    fn test_process_values() {
        let processor = DataProcessor::new(50.0).unwrap();
        let values = vec![30.0, 60.0, 45.0, 70.0];
        let result = processor.process_values(&values);
        
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.len(), 4);
        
        for (original, processed) in values.iter().zip(processed.iter()) {
            if *original > 50.0 {
                assert!(*processed < *original);
            } else {
                assert!(*processed > *original);
            }
        }
    }

    #[test]
    fn test_empty_data() {
        let processor = DataProcessor::new(50.0).unwrap();
        let result = processor.process_values(&[]);
        assert!(result.is_err());
        
        if let Err(e) = result {
            match e {
                DataError::EmptyData => assert!(true),
                _ => panic!("Wrong error type"),
            }
        }
    }
}
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

    pub fn add_dataset(&mut self, name: &str, values: Vec<f64>) -> Result<(), String> {
        if name.is_empty() {
            return Err("Dataset name cannot be empty".to_string());
        }

        if self.data.contains_key(name) {
            return Err(format!("Dataset '{}' already exists", name));
        }

        self.data.insert(name.to_string(), values);
        Ok(())
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn validate_data(&self) -> Vec<String> {
        let mut errors = Vec::new();

        for rule in &self.validation_rules {
            if let Some(data) = self.data.get(&rule.field_name) {
                if rule.required && data.is_empty() {
                    errors.push(format!("Field '{}' is required but empty", rule.field_name));
                    continue;
                }

                for (index, &value) in data.iter().enumerate() {
                    if value < rule.min_value || value > rule.max_value {
                        errors.push(format!(
                            "Value {} at index {} in field '{}' is out of range [{}, {}]",
                            value, index, rule.field_name, rule.min_value, rule.max_value
                        ));
                    }
                }
            } else if rule.required {
                errors.push(format!("Required field '{}' not found", rule.field_name));
            }
        }

        errors
    }

    pub fn normalize_data(&mut self) -> Result<(), String> {
        let mut normalized_data = HashMap::new();

        for (name, values) in &self.data {
            if values.is_empty() {
                normalized_data.insert(name.clone(), Vec::new());
                continue;
            }

            let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

            if (max - min).abs() < f64::EPSILON {
                normalized_data.insert(name.clone(), vec![0.0; values.len()]);
                continue;
            }

            let normalized: Vec<f64> = values
                .iter()
                .map(|&v| (v - min) / (max - min))
                .collect();

            normalized_data.insert(name.clone(), normalized);
        }

        self.data = normalized_data;
        Ok(())
    }

    pub fn calculate_statistics(&self, dataset_name: &str) -> Option<Statistics> {
        self.data.get(dataset_name).map(|values| {
            if values.is_empty() {
                return Statistics::empty();
            }

            let sum: f64 = values.iter().sum();
            let mean = sum / values.len() as f64;

            let variance: f64 = values
                .iter()
                .map(|&v| (v - mean).powi(2))
                .sum::<f64>() / values.len() as f64;

            let std_dev = variance.sqrt();

            let sorted_values = {
                let mut sorted = values.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                sorted
            };

            let median = if values.len() % 2 == 0 {
                let mid = values.len() / 2;
                (sorted_values[mid - 1] + sorted_values[mid]) / 2.0
            } else {
                sorted_values[values.len() / 2]
            };

            Statistics {
                count: values.len(),
                mean,
                median,
                std_dev,
                min: *sorted_values.first().unwrap(),
                max: *sorted_values.last().unwrap(),
            }
        })
    }

    pub fn get_data(&self, name: &str) -> Option<&Vec<f64>> {
        self.data.get(name)
    }

    pub fn list_datasets(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }
}

pub struct Statistics {
    pub count: usize,
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
}

impl Statistics {
    fn empty() -> Self {
        Statistics {
            count: 0,
            mean: 0.0,
            median: 0.0,
            std_dev: 0.0,
            min: 0.0,
            max: 0.0,
        }
    }

    pub fn format_report(&self) -> String {
        format!(
            "Statistics:\n\
             Count: {}\n\
             Mean: {:.4}\n\
             Median: {:.4}\n\
             Std Dev: {:.4}\n\
             Min: {:.4}\n\
             Max: {:.4}",
            self.count, self.mean, self.median, self.std_dev, self.min, self.max
        )
    }
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
    fn test_add_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.add_dataset("temperatures", vec![20.5, 22.1, 19.8, 23.4]);
        assert!(result.is_ok());
        assert_eq!(processor.list_datasets(), vec!["temperatures"]);
    }

    #[test]
    fn test_duplicate_dataset() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("data", vec![1.0, 2.0]).unwrap();
        let result = processor.add_dataset("data", vec![3.0, 4.0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("scores", vec![85.0, 92.0, 78.0, 105.0]).unwrap();

        let rule = ValidationRule::new("scores", 0.0, 100.0, true);
        processor.add_validation_rule(rule);

        let errors = processor.validate_data();
        assert!(errors.len() > 0);
        assert!(errors[0].contains("out of range"));
    }

    #[test]
    fn test_normalization() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("values", vec![10.0, 20.0, 30.0, 40.0]).unwrap();

        processor.normalize_data().unwrap();
        let normalized = processor.get_data("values").unwrap();

        assert_eq!(normalized.len(), 4);
        assert!((normalized[0] - 0.0).abs() < 0.001);
        assert!((normalized[3] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_statistics() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("numbers", vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();

        let stats = processor.calculate_statistics("numbers").unwrap();
        
        assert_eq!(stats.count, 5);
        assert!((stats.mean - 3.0).abs() < 0.001);
        assert!((stats.median - 3.0).abs() < 0.001);
        assert!((stats.std_dev - 1.4142).abs() < 0.001);
    }
}