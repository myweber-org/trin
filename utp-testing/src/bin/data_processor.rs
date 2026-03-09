
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.is_empty() {
                continue;
            }

            if self.has_header && line_number == 0 {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if fields.iter().any(|f| f.is_empty()) {
                return Err(format!("Empty field detected at line {}", line_number + 1).into());
            }

            records.push(fields);
        }

        if records.is_empty() {
            return Err("No valid records found in file".into());
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<(), Box<dyn Error>> {
        if records.is_empty() {
            return Err("No records to validate".into());
        }

        let expected_len = records[0].len();
        
        for (idx, record) in records.iter().enumerate() {
            if record.len() != expected_len {
                return Err(format!("Record {} has {} fields, expected {}", 
                    idx + 1, record.len(), expected_len).into());
            }
        }

        Ok(())
    }

    pub fn calculate_column_stats(&self, records: &[Vec<String>]) -> Vec<(usize, usize)> {
        if records.is_empty() {
            return Vec::new();
        }

        let num_columns = records[0].len();
        let mut stats = vec![(0, 0); num_columns];

        for record in records {
            for (col_idx, field) in record.iter().enumerate() {
                let (sum, count) = stats[col_idx];
                if let Ok(num) = field.parse::<usize>() {
                    stats[col_idx] = (sum + num, count + 1);
                }
            }
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_validate_records() {
        let records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        assert!(processor.validate_records(&records).is_ok());
    }

    #[test]
    fn test_calculate_column_stats() {
        let records = vec![
            vec!["10".to_string(), "20".to_string()],
            vec!["30".to_string(), "40".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let stats = processor.calculate_column_stats(&records);
        
        assert_eq!(stats[0], (40, 2));
        assert_eq!(stats[1], (60, 2));
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    tags: Vec<String>,
    metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64) -> Self {
        Self {
            id,
            name,
            value,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        Ok(())
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Vec<DataRecord> {
    records
        .iter_mut()
        .filter(|record| record.validate().is_ok())
        .map(|record| {
            let mut processed = DataRecord::new(record.id, record.name.clone(), record.value);
            processed.tags = record.tags.clone();
            processed.metadata = record.metadata.clone();
            processed.value = (record.value * 100.0).round() / 100.0;
            processed
        })
        .collect()
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if records.is_empty() {
        return stats;
    }

    let count = records.len() as f64;
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let avg = sum / count;
    let max = records.iter().map(|r| r.value).fold(f64::NEG_INFINITY, f64::max);
    let min = records.iter().map(|r| r.value).fold(f64::INFINITY, f64::min);

    stats.insert("count".to_string(), count);
    stats.insert("sum".to_string(), sum);
    stats.insert("average".to_string(), avg);
    stats.insert("maximum".to_string(), max);
    stats.insert("minimum".to_string(), min);

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, "Test".to_string(), 42.5);
        assert!(valid_record.validate().is_ok());

        let invalid_name = DataRecord::new(2, "".to_string(), 42.5);
        assert!(invalid_name.validate().is_err());

        let invalid_value = DataRecord::new(3, "Test".to_string(), -5.0);
        assert!(invalid_value.validate().is_err());
    }

    #[test]
    fn test_process_records() {
        let mut records = vec![
            DataRecord::new(1, "Record1".to_string(), 123.456),
            DataRecord::new(2, "".to_string(), 78.9),
            DataRecord::new(3, "Record3".to_string(), -10.0),
        ];

        let processed = process_records(&mut records);
        assert_eq!(processed.len(), 1);
        assert_eq!(processed[0].value, 123.46);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord::new(1, "A".to_string(), 10.0),
            DataRecord::new(2, "B".to_string(), 20.0),
            DataRecord::new(3, "C".to_string(), 30.0),
        ];

        let stats = calculate_statistics(&records);
        assert_eq!(stats["count"], 3.0);
        assert_eq!(stats["sum"], 60.0);
        assert_eq!(stats["average"], 20.0);
        assert_eq!(stats["maximum"], 30.0);
        assert_eq!(stats["minimum"], 10.0);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
    validation_rules: HashMap<String, ValidationRule>,
}

pub struct ValidationRule {
    min_value: Option<f64>,
    max_value: Option<f64>,
    required: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: HashMap::new(),
            validation_rules: HashMap::new(),
        }
    }

    pub fn add_dataset(&mut self, name: String, values: Vec<f64>) -> Result<(), String> {
        if self.data.contains_key(&name) {
            return Err(format!("Dataset '{}' already exists", name));
        }
        
        if let Some(rule) = self.validation_rules.get(&name) {
            if rule.required && values.is_empty() {
                return Err(format!("Dataset '{}' cannot be empty", name));
            }
            
            for &value in &values {
                if let Some(min) = rule.min_value {
                    if value < min {
                        return Err(format!("Value {} below minimum {}", value, min));
                    }
                }
                
                if let Some(max) = rule.max_value {
                    if value > max {
                        return Err(format!("Value {} above maximum {}", value, max));
                    }
                }
            }
        }
        
        self.data.insert(name, values);
        Ok(())
    }

    pub fn set_validation_rule(&mut self, dataset_name: String, rule: ValidationRule) {
        self.validation_rules.insert(dataset_name, rule);
    }

    pub fn calculate_statistics(&self, dataset_name: &str) -> Option<Statistics> {
        self.data.get(dataset_name).map(|values| {
            let count = values.len();
            let sum: f64 = values.iter().sum();
            let mean = if count > 0 { sum / count as f64 } else { 0.0 };
            
            let variance = if count > 1 {
                let squared_diff: f64 = values.iter()
                    .map(|&x| (x - mean).powi(2))
                    .sum();
                squared_diff / (count - 1) as f64
            } else {
                0.0
            };
            
            Statistics {
                count,
                mean,
                variance,
                min: values.iter().copied().fold(f64::INFINITY, f64::min),
                max: values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            }
        })
    }

    pub fn normalize_data(&self, dataset_name: &str) -> Option<Vec<f64>> {
        self.data.get(dataset_name).and_then(|values| {
            if values.is_empty() {
                return None;
            }
            
            let stats = self.calculate_statistics(dataset_name)?;
            if stats.max - stats.min == 0.0 {
                return Some(vec![0.0; values.len()]);
            }
            
            Some(values.iter()
                .map(|&x| (x - stats.min) / (stats.max - stats.min))
                .collect())
        })
    }

    pub fn merge_datasets(&self, names: &[&str]) -> Option<Vec<f64>> {
        let mut result = Vec::new();
        
        for &name in names {
            if let Some(values) = self.data.get(name) {
                result.extend(values);
            } else {
                return None;
            }
        }
        
        Some(result)
    }
}

pub struct Statistics {
    pub count: usize,
    pub mean: f64,
    pub variance: f64,
    pub min: f64,
    pub max: f64,
}

impl ValidationRule {
    pub fn new() -> Self {
        ValidationRule {
            min_value: None,
            max_value: None,
            required: false,
        }
    }

    pub fn with_min(mut self, min: f64) -> Self {
        self.min_value = Some(min);
        self
    }

    pub fn with_max(mut self, max: f64) -> Self {
        self.max_value = Some(max);
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_validate_dataset() {
        let mut processor = DataProcessor::new();
        let rule = ValidationRule::new()
            .with_min(0.0)
            .with_max(100.0)
            .required();
        
        processor.set_validation_rule("temperatures".to_string(), rule);
        
        let result = processor.add_dataset(
            "temperatures".to_string(),
            vec![25.5, 30.0, 15.2, 99.9]
        );
        
        assert!(result.is_ok());
        assert_eq!(processor.data.len(), 1);
    }

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("test".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0])
            .unwrap();
        
        let stats = processor.calculate_statistics("test").unwrap();
        
        assert_eq!(stats.count, 5);
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
    }
}