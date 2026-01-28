
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

    pub fn process_dataset(&mut self, dataset_name: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        for rule in &self.validation_rules {
            if rule.required && data.iter().any(|&x| x.is_nan()) {
                return Err(format!("Field '{}' contains invalid values", rule.field_name));
            }
        }

        let processed_data: Vec<f64> = data
            .iter()
            .map(|&value| {
                let mut transformed = value;
                
                for rule in &self.validation_rules {
                    if value < rule.min_value {
                        transformed = rule.min_value;
                    } else if value > rule.max_value {
                        transformed = rule.max_value;
                    }
                }
                
                transformed * 1.05
            })
            .collect();

        self.cache.insert(dataset_name.to_string(), processed_data.clone());
        
        Ok(processed_data)
    }

    pub fn get_cached_data(&self, dataset_name: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_name)
    }

    pub fn calculate_statistics(&self, dataset_name: &str) -> Option<DatasetStatistics> {
        self.cache.get(dataset_name).map(|data| {
            let sum: f64 = data.iter().sum();
            let count = data.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = data.iter()
                .map(|&value| (value - mean).powi(2))
                .sum::<f64>() / count;
            
            DatasetStatistics {
                mean,
                variance,
                min: *data.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0),
                max: *data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0),
                count: data.len(),
            }
        })
    }
}

pub struct DatasetStatistics {
    pub mean: f64,
    pub variance: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
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
        processor.add_validation_rule(ValidationRule::new("temperature", -50.0, 150.0, true));
        
        let test_data = vec![25.0, 30.0, 35.0, 40.0];
        let result = processor.process_dataset("test_set", &test_data);
        
        assert!(result.is_ok());
        assert_eq!(processor.get_cached_data("test_set").unwrap().len(), 4);
    }

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        let test_data = vec![10.0, 20.0, 30.0, 40.0];
        
        processor.process_dataset("stats_test", &test_data).unwrap();
        let stats = processor.calculate_statistics("stats_test").unwrap();
        
        assert_eq!(stats.mean, 26.25);
        assert_eq!(stats.min, 10.5);
        assert_eq!(stats.max, 42.0);
        assert_eq!(stats.count, 4);
    }
}
use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
}

pub fn process_csv_file(input_path: &str, output_path: &str) -> Result<usize, Box<dyn Error>> {
    let input_path = Path::new(input_path);
    let mut reader = Reader::from_path(input_path)?;
    
    let output_path = Path::new(output_path);
    let mut writer = Writer::from_path(output_path)?;
    
    let mut valid_count = 0;
    
    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.is_valid() {
            writer.serialize(&record)?;
            valid_count += 1;
        }
    }
    
    writer.flush()?;
    Ok(valid_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_valid_record() {
        let record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 42.5,
            active: true,
        };
        assert!(record.is_valid());
    }
    
    #[test]
    fn test_invalid_record() {
        let record = Record {
            id: 2,
            name: "".to_string(),
            value: -10.0,
            active: false,
        };
        assert!(!record.is_valid());
    }
    
    #[test]
    fn test_csv_processing() -> Result<(), Box<dyn Error>> {
        let input_data = "id,name,value,active\n1,Alice,100.5,true\n2,Bob,-50.0,false\n3,,75.3,true";
        
        let input_file = NamedTempFile::new()?;
        fs::write(&input_file, input_data)?;
        
        let output_file = NamedTempFile::new()?;
        
        let valid_count = process_csv_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        )?;
        
        assert_eq!(valid_count, 1);
        
        let output_content = fs::read_to_string(output_file.path())?;
        assert!(output_content.contains("Alice"));
        assert!(!output_content.contains("Bob"));
        assert!(!output_content.contains(",,"));
        
        Ok(())
    }
}