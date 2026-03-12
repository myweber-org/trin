
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

    pub fn validate_data(&self) -> Vec<ValidationResult> {
        let mut results = Vec::new();

        for rule in &self.validation_rules {
            if let Some(data_values) = self.data.get(&rule.field_name) {
                if rule.required && data_values.is_empty() {
                    results.push(ValidationResult::new(
                        &rule.field_name,
                        false,
                        "Required field is empty".to_string(),
                    ));
                    continue;
                }

                for (index, &value) in data_values.iter().enumerate() {
                    if value < rule.min_value || value > rule.max_value {
                        results.push(ValidationResult::new(
                            &rule.field_name,
                            false,
                            format!(
                                "Value {} at index {} is outside valid range [{}, {}]",
                                value, index, rule.min_value, rule.max_value
                            ),
                        ));
                    }
                }
            } else if rule.required {
                results.push(ValidationResult::new(
                    &rule.field_name,
                    false,
                    "Required field not found in dataset".to_string(),
                ));
            }
        }

        if results.is_empty() {
            results.push(ValidationResult::new(
                "all",
                true,
                "All validations passed".to_string(),
            ));
        }

        results
    }

    pub fn transform_data(&self, transform_type: TransformType) -> HashMap<String, Vec<f64>> {
        let mut transformed = HashMap::new();

        for (name, values) in &self.data {
            let transformed_values: Vec<f64> = match transform_type {
                TransformType::Normalize => {
                    if let Some(&max) = values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()) {
                        if max != 0.0 {
                            values.iter().map(|&v| v / max).collect()
                        } else {
                            values.clone()
                        }
                    } else {
                        values.clone()
                    }
                }
                TransformType::Standardize => {
                    let mean = values.iter().sum::<f64>() / values.len() as f64;
                    let variance = values
                        .iter()
                        .map(|&v| (v - mean).powi(2))
                        .sum::<f64>()
                        / values.len() as f64;
                    let std_dev = variance.sqrt();

                    if std_dev != 0.0 {
                        values.iter().map(|&v| (v - mean) / std_dev).collect()
                    } else {
                        values.clone()
                    }
                }
                TransformType::LogTransform => {
                    values
                        .iter()
                        .map(|&v| if v > 0.0 { v.ln() } else { v })
                        .collect()
                }
            };

            transformed.insert(name.clone(), transformed_values);
        }

        transformed
    }

    pub fn calculate_statistics(&self) -> HashMap<String, DatasetStatistics> {
        let mut stats = HashMap::new();

        for (name, values) in &self.data {
            if values.is_empty() {
                stats.insert(
                    name.clone(),
                    DatasetStatistics {
                        count: 0,
                        mean: 0.0,
                        min: 0.0,
                        max: 0.0,
                        sum: 0.0,
                    },
                );
                continue;
            }

            let count = values.len();
            let sum: f64 = values.iter().sum();
            let mean = sum / count as f64;
            let min = *values
                .iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();
            let max = *values
                .iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();

            stats.insert(
                name.clone(),
                DatasetStatistics {
                    count,
                    mean,
                    min,
                    max,
                    sum,
                },
            );
        }

        stats
    }
}

pub struct ValidationResult {
    field_name: String,
    is_valid: bool,
    message: String,
}

impl ValidationResult {
    pub fn new(field_name: &str, is_valid: bool, message: String) -> Self {
        ValidationResult {
            field_name: field_name.to_string(),
            is_valid,
            message,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.is_valid
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn field_name(&self) -> &str {
        &self.field_name
    }
}

pub struct DatasetStatistics {
    pub count: usize,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub sum: f64,
}

pub enum TransformType {
    Normalize,
    Standardize,
    LogTransform,
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
        let result = processor.add_dataset("temperatures", vec![20.5, 22.3, 19.8, 21.7]);
        assert!(result.is_ok());
        assert!(processor.data.contains_key("temperatures"));
    }

    #[test]
    fn test_duplicate_dataset() {
        let mut processor = DataProcessor::new();
        processor
            .add_dataset("temperatures", vec![20.5, 22.3])
            .unwrap();
        let result = processor.add_dataset("temperatures", vec![19.8, 21.7]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation() {
        let mut processor = DataProcessor::new();
        processor
            .add_dataset("temperatures", vec![20.5, 22.3, 19.8, 21.7])
            .unwrap();

        let rule = ValidationRule::new("temperatures", 15.0, 30.0, true);
        processor.add_validation_rule(rule);

        let results = processor.validate_data();
        assert!(results[0].is_valid());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        processor
            .add_dataset("temperatures", vec![20.0, 25.0, 30.0])
            .unwrap();

        let stats = processor.calculate_statistics();
        let temp_stats = stats.get("temperatures").unwrap();

        assert_eq!(temp_stats.count, 3);
        assert_eq!(temp_stats.mean, 25.0);
        assert_eq!(temp_stats.min, 20.0);
        assert_eq!(temp_stats.max, 30.0);
        assert_eq!(temp_stats.sum, 75.0);
    }

    #[test]
    fn test_data_transformation() {
        let mut processor = DataProcessor::new();
        processor
            .add_dataset("values", vec![1.0, 2.0, 3.0, 4.0])
            .unwrap();

        let normalized = processor.transform_data(TransformType::Normalize);
        let normalized_values = normalized.get("values").unwrap();

        assert_eq!(normalized_values, &vec![0.25, 0.5, 0.75, 1.0]);
    }
}use csv::Reader;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

fn process_csv_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut rdr = Reader::from_reader(file);
    
    let mut records: Vec<Record> = Vec::new();
    
    for result in rdr.deserialize() {
        let record: Record = result?;
        if record.active && record.value > 50.0 {
            records.push(record);
        }
    }
    
    let output_file = File::create(output_path)?;
    let mut wtr = csv::Writer::from_writer(output_file);
    
    for record in records {
        wtr.serialize(record)?;
    }
    
    wtr.flush()?;
    Ok(())
}

fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    if count == 0.0 {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (sum, mean, std_dev)
}

fn validate_record(record: &Record) -> bool {
    !record.name.is_empty() && record.value >= 0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: "Test1".to_string(), value: 100.0, active: true },
            Record { id: 2, name: "Test2".to_string(), value: 200.0, active: true },
        ];
        
        let (sum, mean, std_dev) = calculate_statistics(&records);
        
        assert_eq!(sum, 300.0);
        assert_eq!(mean, 150.0);
        assert!(std_dev > 0.0);
    }
    
    #[test]
    fn test_record_validation() {
        let valid_record = Record {
            id: 1,
            name: "Valid".to_string(),
            value: 100.0,
            active: true,
        };
        
        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            value: -10.0,
            active: true,
        };
        
        assert!(validate_record(&valid_record));
        assert!(!validate_record(&invalid_record));
    }
}