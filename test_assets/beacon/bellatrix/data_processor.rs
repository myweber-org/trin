
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
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.value >= 0.0 && !r.name.is_empty())
            .collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&Record>> {
        let mut map = std::collections::HashMap::new();
        
        for record in &self.records {
            map.entry(record.category.clone())
               .or_insert_with(Vec::new)
               .push(record);
        }
        
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_data_processing() {
        let csv_data = "id,name,value,category\n1,ItemA,10.5,Category1\n2,ItemB,15.3,Category2\n";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();
        
        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        assert_eq!(processor.calculate_total(), 25.8);
        
        let valid_records = processor.validate_records();
        assert_eq!(valid_records.len(), 2);
        
        let grouped = processor.group_by_category();
        assert_eq!(grouped.len(), 2);
    }
}
use csv;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, active: bool) -> Self {
        Self {
            id,
            name,
            value,
            active,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }

    pub fn calculate_adjusted_value(&self) -> f64 {
        if self.active {
            self.value * 1.1
        } else {
            self.value * 0.9
        }
    }
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn add_record(&mut self, record: Record) {
        self.records.push(record);
    }

    pub fn filter_valid_records(&self) -> Vec<&Record> {
        self.records.iter().filter(|r| r.is_valid()).collect()
    }

    pub fn calculate_total_adjusted_value(&self) -> f64 {
        self.records
            .iter()
            .map(|r| r.calculate_adjusted_value())
            .sum()
    }

    pub fn save_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        let mut wtr = csv::Writer::from_writer(file);

        for record in &self.records {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    pub fn get_statistics(&self) -> (usize, f64, f64) {
        let count = self.records.len();
        let total_value: f64 = self.records.iter().map(|r| r.value).sum();
        let avg_value = if count > 0 {
            total_value / count as f64
        } else {
            0.0
        };

        (count, total_value, avg_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "Test".to_string(), 100.0, true);
        assert!(valid_record.is_valid());

        let invalid_record = Record::new(2, "".to_string(), -50.0, false);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_adjusted_value_calculation() {
        let active_record = Record::new(1, "Active".to_string(), 100.0, true);
        assert_eq!(active_record.calculate_adjusted_value(), 110.0);

        let inactive_record = Record::new(2, "Inactive".to_string(), 100.0, false);
        assert_eq!(inactive_record.calculate_adjusted_value(), 90.0);
    }

    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();
        
        processor.add_record(Record::new(1, "First".to_string(), 50.0, true));
        processor.add_record(Record::new(2, "Second".to_string(), 75.0, false));
        
        let valid_records = processor.filter_valid_records();
        assert_eq!(valid_records.len(), 2);
        
        let total_adjusted = processor.calculate_total_adjusted_value();
        assert_eq!(total_adjusted, 50.0 * 1.1 + 75.0 * 0.9);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.0, 2);
        assert_eq!(stats.1, 125.0);
        assert_eq!(stats.2, 62.5);
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let mut processor = DataProcessor::new();
        processor.add_record(Record::new(1, "Test".to_string(), 100.0, true));
        
        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();
        
        processor.save_to_csv(path)?;
        
        let mut new_processor = DataProcessor::new();
        new_processor.load_from_csv(path)?;
        
        assert_eq!(new_processor.records.len(), 1);
        Ok(())
    }
}