use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    value: f64,
    category: String,
}

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    fn calculate_statistics(&self) -> (f64, f64, f64) {
        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        
        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = if count > 0.0 { sum / count } else { 0.0 };
        
        let variance: f64 = if count > 0.0 {
            values.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / count
        } else {
            0.0
        };
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }

    fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    fn get_max_value(&self) -> Option<f64> {
        self.records.iter().map(|r| r.value).max_by(|a, b| a.partial_cmp(b).unwrap())
    }

    fn get_min_value(&self) -> Option<f64> {
        self.records.iter().map(|r| r.value).min_by(|a, b| a.partial_cmp(b).unwrap())
    }
}

fn process_data_file(input_path: &str) -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    processor.load_from_csv(input_path)?;
    
    let (mean, variance, std_dev) = processor.calculate_statistics();
    println!("Statistics - Mean: {:.2}, Variance: {:.2}, Std Dev: {:.2}", mean, variance, std_dev);
    
    if let Some(max) = processor.get_max_value() {
        println!("Maximum value: {:.2}", max);
    }
    
    if let Some(min) = processor.get_min_value() {
        println!("Minimum value: {:.2}", min);
    }
    
    let categories = vec!["A", "B", "C"];
    for category in categories {
        let filtered = processor.filter_by_category(category);
        println!("Category {}: {} records", category, filtered.len());
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/sample_data.csv";
    process_data_file(input_file)
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
        let mut results = Vec::with_capacity(dataset.len());
        
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
                return Err(format!("Required field '{}' not found", rule.field_name));
            }
        }
        Ok(())
    }

    fn transform_record(&self, record: &HashMap<String, f64>) -> ProcessedRecord {
        let mut values = Vec::new();
        let mut sum = 0.0;
        
        for (key, &value) in record {
            let transformed = match key.as_str() {
                "temperature" => (value - 32.0) * 5.0 / 9.0,
                "pressure" => value * 100.0,
                "humidity" => value / 100.0,
                _ => value,
            };
            values.push(transformed);
            sum += transformed;
        }
        
        let average = if !values.is_empty() {
            sum / values.len() as f64
        } else {
            0.0
        };
        
        ProcessedRecord { values, average }
    }

    fn cache_record(&mut self, index: usize, values: &[f64]) {
        self.cache.insert(format!("record_{}", index), values.to_vec());
    }

    pub fn get_cached_record(&self, index: usize) -> Option<&Vec<f64>> {
        self.cache.get(&format!("record_{}", index))
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

pub struct ProcessedRecord {
    pub values: Vec<f64>,
    pub average: f64,
}

impl ValidationRule {
    pub fn new(field_name: String, min_value: f64, max_value: f64, required: bool) -> Self {
        ValidationRule {
            field_name,
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
        processor.add_validation_rule(ValidationRule::new("temperature".to_string(), -50.0, 150.0, true));
        
        let mut record = HashMap::new();
        record.insert("temperature".to_string(), 68.0);
        record.insert("humidity".to_string(), 65.0);
        
        let dataset = vec![record];
        let result = processor.process_dataset(&dataset);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(ValidationRule::new("pressure".to_string(), 0.0, 100.0, true));
        
        let mut record = HashMap::new();
        record.insert("pressure".to_string(), 150.0);
        
        let dataset = vec![record];
        let result = processor.process_dataset(&dataset);
        
        assert!(result.is_err());
    }
}