use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut count = 0;
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }
            
            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };
            
            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };
            
            let category = parts[2].trim().to_string();
            
            if !self.validate_record(&category, value) {
                continue;
            }
            
            self.records.push(DataRecord { id, value, category });
            count += 1;
        }
        
        Ok(count)
    }
    
    fn validate_record(&self, category: &str, value: f64) -> bool {
        !category.is_empty() && value >= 0.0 && value <= 1000.0
    }
    
    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }
    
    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }
    
    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average().unwrap_or(0.0);
        
        (min, max, avg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,100.5,TypeA").unwrap();
        writeln!(temp_file, "2,200.3,TypeB").unwrap();
        writeln!(temp_file, "3,300.7,TypeA").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 200.5).abs() < 0.01);
        
        let type_a_records = processor.filter_by_category("TypeA");
        assert_eq!(type_a_records.len(), 2);
        
        let stats = processor.get_statistics();
        assert!((stats.0 - 100.5).abs() < 0.01);
        assert!((stats.1 - 300.7).abs() < 0.01);
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
    pub category: String,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidValue,
    MissingField,
    DuplicateRecord,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::InvalidValue => write!(f, "Invalid numeric value"),
            DataError::MissingField => write!(f, "Required field is missing"),
            DataError::DuplicateRecord => write!(f, "Duplicate record detected"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    category_stats: HashMap<String, CategoryStats>,
}

#[derive(Debug, Clone)]
pub struct CategoryStats {
    pub count: usize,
    pub total_value: f64,
    pub average_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
            category_stats: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }

        if record.value < 0.0 || record.value.is_nan() {
            return Err(DataError::InvalidValue);
        }

        if record.name.is_empty() || record.category.is_empty() {
            return Err(DataError::MissingField);
        }

        if self.records.contains_key(&record.id) {
            return Err(DataError::DuplicateRecord);
        }

        self.records.insert(record.id, record.clone());
        self.update_category_stats(&record);

        Ok(())
    }

    fn update_category_stats(&mut self, record: &DataRecord) {
        let stats = self.category_stats
            .entry(record.category.clone())
            .or_insert(CategoryStats {
                count: 0,
                total_value: 0.0,
                average_value: 0.0,
            });

        stats.count += 1;
        stats.total_value += record.value;
        stats.average_value = stats.total_value / stats.count as f64;
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn get_category_stats(&self, category: &str) -> Option<&CategoryStats> {
        self.category_stats.get(category)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.values().map(|record| record.value).sum()
    }

    pub fn get_top_records(&self, limit: usize) -> Vec<&DataRecord> {
        let mut records: Vec<&DataRecord> = self.records.values().collect();
        records.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap());
        records.into_iter().take(limit).collect()
    }

    pub fn validate_all_records(&self) -> Vec<DataError> {
        let mut errors = Vec::new();

        for record in self.records.values() {
            if record.id == 0 {
                errors.push(DataError::InvalidId);
            }
            if record.value < 0.0 || record.value.is_nan() {
                errors.push(DataError::InvalidValue);
            }
            if record.name.is_empty() || record.category.is_empty() {
                errors.push(DataError::MissingField);
            }
        }

        errors
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
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
            value: 100.0,
            category: "Test".to_string(),
        };

        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.records.len(), 1);
    }

    #[test]
    fn test_add_duplicate_record() {
        let mut processor = DataProcessor::new();
        let record1 = DataRecord {
            id: 1,
            name: "Record 1".to_string(),
            value: 100.0,
            category: "Test".to_string(),
        };

        let record2 = DataRecord {
            id: 1,
            name: "Record 2".to_string(),
            value: 200.0,
            category: "Test".to_string(),
        };

        assert!(processor.add_record(record1).is_ok());
        assert!(matches!(processor.add_record(record2), Err(DataError::DuplicateRecord)));
    }

    #[test]
    fn test_category_stats() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord {
                id: 1,
                name: "Record 1".to_string(),
                value: 100.0,
                category: "CategoryA".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Record 2".to_string(),
                value: 200.0,
                category: "CategoryA".to_string(),
            },
            DataRecord {
                id: 3,
                name: "Record 3".to_string(),
                value: 300.0,
                category: "CategoryB".to_string(),
            },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        let stats_a = processor.get_category_stats("CategoryA").unwrap();
        assert_eq!(stats_a.count, 2);
        assert_eq!(stats_a.total_value, 300.0);
        assert_eq!(stats_a.average_value, 150.0);

        let stats_b = processor.get_category_stats("CategoryB").unwrap();
        assert_eq!(stats_b.count, 1);
        assert_eq!(stats_b.total_value, 300.0);
        assert_eq!(stats_b.average_value, 300.0);
    }

    #[test]
    fn test_filter_records() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord {
                id: 1,
                name: "Record 1".to_string(),
                value: 100.0,
                category: "CategoryA".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Record 2".to_string(),
                value: 200.0,
                category: "CategoryB".to_string(),
            },
            DataRecord {
                id: 3,
                name: "Record 3".to_string(),
                value: 300.0,
                category: "CategoryA".to_string(),
            },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        let category_a_records = processor.filter_by_category("CategoryA");
        assert_eq!(category_a_records.len(), 2);
        
        let category_b_records = processor.filter_by_category("CategoryB");
        assert_eq!(category_b_records.len(), 1);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub field_name: String,
    pub min_value: f64,
    pub max_value: f64,
    pub required: bool,
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

    pub fn process_data(&mut self, dataset: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, String> {
        if dataset.is_empty() {
            return Err("Empty dataset provided".to_string());
        }

        let mut processed = Vec::with_capacity(dataset.len());
        
        for (index, row) in dataset.iter().enumerate() {
            match self.validate_row(row) {
                Ok(validated_row) => {
                    let transformed = self.transform_row(&validated_row);
                    processed.push(transformed);
                    self.cache.insert(format!("row_{}", index), validated_row);
                }
                Err(e) => return Err(format!("Validation failed at row {}: {}", index, e)),
            }
        }

        Ok(processed)
    }

    fn validate_row(&self, row: &[f64]) -> Result<Vec<f64>, String> {
        if row.len() != self.validation_rules.len() {
            return Err(format!(
                "Row length {} doesn't match validation rules count {}",
                row.len(),
                self.validation_rules.len()
            ));
        }

        let mut validated = Vec::with_capacity(row.len());
        
        for (i, (&value, rule)) in row.iter().zip(self.validation_rules.iter()).enumerate() {
            if rule.required && value.is_nan() {
                return Err(format!("Required field {} is NaN at position {}", rule.field_name, i));
            }
            
            if !value.is_nan() && (value < rule.min_value || value > rule.max_value) {
                return Err(format!(
                    "Field {} value {} out of range [{}, {}] at position {}",
                    rule.field_name, value, rule.min_value, rule.max_value, i
                ));
            }
            
            validated.push(value);
        }

        Ok(validated)
    }

    fn transform_row(&self, row: &[f64]) -> Vec<f64> {
        row.iter()
            .map(|&x| {
                if x.is_nan() {
                    0.0
                } else {
                    x * 2.0
                }
            })
            .collect()
    }

    pub fn get_cached_row(&self, key: &str) -> Option<&Vec<f64>> {
        self.cache.get(key)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn get_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        stats.insert("cache_size".to_string(), self.cache.len() as f64);
        stats.insert("validation_rules_count".to_string(), self.validation_rules.len() as f64);
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor_validation() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(ValidationRule {
            field_name: "temperature".to_string(),
            min_value: -50.0,
            max_value: 100.0,
            required: true,
        });

        let valid_data = vec![vec![25.0]];
        let result = processor.process_data(&valid_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_data_handling() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(ValidationRule {
            field_name: "pressure".to_string(),
            min_value: 0.0,
            max_value: 10.0,
            required: true,
        });

        let invalid_data = vec![vec![15.0]];
        let result = processor.process_data(&invalid_data);
        assert!(result.is_err());
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
        }
    }

    pub fn process_numeric_data(&mut self, key: &str, values: &[f64]) -> Result<Vec<f64>, String> {
        if values.is_empty() {
            return Err("Empty data provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let validated = self.validate_data(values)?;
        let processed = self.transform_data(&validated);
        
        self.cache.insert(key.to_string(), processed.clone());
        Ok(processed)
    }

    fn validate_data(&self, values: &[f64]) -> Result<Vec<f64>, String> {
        let mut result = Vec::with_capacity(values.len());
        
        for &value in values {
            if value.is_nan() || value.is_infinite() {
                return Err(format!("Invalid numeric value detected: {}", value));
            }
            if value < 0.0 {
                return Err("Negative values not allowed".to_string());
            }
            result.push(value);
        }
        
        Ok(result)
    }

    fn transform_data(&self, values: &[f64]) -> Vec<f64> {
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        
        values.iter()
            .map(|&x| (x - mean).abs())
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_validation() {
        let processor = DataProcessor::new();
        let valid_data = vec![1.0, 2.0, 3.0];
        let invalid_data = vec![1.0, f64::NAN, 3.0];
        
        assert!(processor.validate_data(&valid_data).is_ok());
        assert!(processor.validate_data(&invalid_data).is_err());
    }

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0];
        
        let result = processor.process_numeric_data("test", &data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), 3);
        assert!(processed[0] > 0.0);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Self {
        let valid = value >= 0.0 && !category.is_empty();
        DataRecord {
            id,
            value,
            category: category.to_string(),
            valid,
        }
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let record = DataRecord::new(id, value, parts[2]);
            self.records.push(record);
            count += 1;
        }

        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.valid).collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.filter_valid();
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&DataRecord>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            if record.valid {
                groups
                    .entry(record.category.clone())
                    .or_insert_with(Vec::new)
                    .push(record);
            }
        }
        
        groups
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test");
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
        assert!(record.valid);
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(2, -5.0, "");
        assert!(!record.valid);
    }

    #[test]
    fn test_load_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,42.5,category_a").unwrap();
        writeln!(temp_file, "2,-3.0,category_b").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "3,15.75,category_a").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_file(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.count_records(), 3);
    }

    #[test]
    fn test_filter_valid() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "cat1"));
        processor.records.push(DataRecord::new(2, -5.0, "cat2"));
        processor.records.push(DataRecord::new(3, 20.0, "cat1"));

        let valid = processor.filter_valid();
        assert_eq!(valid.len(), 2);
    }

    #[test]
    fn test_calculate_average() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "cat1"));
        processor.records.push(DataRecord::new(2, 20.0, "cat2"));
        processor.records.push(DataRecord::new(3, 30.0, "cat1"));

        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert_eq!(avg.unwrap(), 20.0);
    }

    #[test]
    fn test_empty_average() {
        let processor = DataProcessor::new();
        let avg = processor.calculate_average();
        assert!(avg.is_none());
    }

    #[test]
    fn test_group_by_category() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "cat1"));
        processor.records.push(DataRecord::new(2, 20.0, "cat2"));
        processor.records.push(DataRecord::new(3, 30.0, "cat1"));
        processor.records.push(DataRecord::new(4, -5.0, "cat1"));

        let groups = processor.group_by_category();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("cat1").unwrap().len(), 2);
        assert_eq!(groups.get("cat2").unwrap().len(), 1);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Self {
        let valid = value >= 0.0 && !category.is_empty();
        DataRecord {
            id,
            value,
            category: category.to_string(),
            valid,
        }
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    stats: ProcessingStats,
}

#[derive(Debug, Default)]
pub struct ProcessingStats {
    pub total_records: usize,
    pub valid_records: usize,
    pub invalid_records: usize,
    pub sum_values: f64,
    pub avg_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            stats: ProcessingStats::default(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut line_count = 0;
        for line in reader.lines() {
            let line = line?;
            line_count += 1;
            
            if line_count == 1 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() < 3 {
                continue;
            }
            
            let id = parts[0].parse().unwrap_or(0);
            let value = parts[1].parse().unwrap_or(0.0);
            let category = parts[2];
            
            let record = DataRecord::new(id, value, category);
            self.records.push(record);
        }
        
        self.calculate_stats();
        Ok(())
    }
    
    fn calculate_stats(&mut self) {
        self.stats.total_records = self.records.len();
        self.stats.valid_records = self.records.iter().filter(|r| r.valid).count();
        self.stats.invalid_records = self.stats.total_records - self.stats.valid_records;
        
        let valid_values: Vec<f64> = self.records
            .iter()
            .filter(|r| r.valid)
            .map(|r| r.value)
            .collect();
            
        self.stats.sum_values = valid_values.iter().sum();
        self.stats.avg_value = if !valid_values.is_empty() {
            self.stats.sum_values / valid_values.len() as f64
        } else {
            0.0
        };
    }
    
    pub fn get_valid_records(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.valid).collect()
    }
    
    pub fn get_stats(&self) -> &ProcessingStats {
        &self.stats
    }
    
    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category && r.valid)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test");
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
        assert!(record.valid);
    }
    
    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(2, -5.0, "");
        assert!(!record.valid);
    }
    
    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,type_a").unwrap();
        writeln!(temp_file, "2,20.3,type_b").unwrap();
        writeln!(temp_file, "3,-5.0,type_a").unwrap();
        
        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        
        let stats = processor.get_stats();
        assert_eq!(stats.total_records, 3);
        assert_eq!(stats.valid_records, 2);
        assert_eq!(stats.invalid_records, 1);
    }
}