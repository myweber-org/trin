use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    metadata: HashMap<String, String>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn load_from_csv(&mut self, filepath: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                self.parse_header(&line);
                continue;
            }
            
            if let Ok(value) = line.trim().parse::<f64>() {
                self.data.push(value);
            }
        }
        
        self.metadata.insert("source".to_string(), filepath.to_string());
        self.metadata.insert("loaded_at".to_string(), chrono::Local::now().to_rfc3339());
        
        Ok(())
    }

    fn parse_header(&mut self, header: &str) {
        let columns: Vec<&str> = header.split(',').collect();
        self.metadata.insert("columns".to_string(), columns.len().to_string());
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.data.is_empty() {
            return stats;
        }

        let sum: f64 = self.data.iter().sum();
        let count = self.data.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        let sorted_data = {
            let mut sorted = self.data.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            sorted
        };

        let median = if count as usize % 2 == 0 {
            let mid = count as usize / 2;
            (sorted_data[mid - 1] + sorted_data[mid]) / 2.0
        } else {
            sorted_data[count as usize / 2]
        };

        stats.insert("mean".to_string(), mean);
        stats.insert("median".to_string(), median);
        stats.insert("variance".to_string(), variance);
        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);

        if let Some(min) = self.data.iter().min_by(|a, b| a.partial_cmp(b).unwrap()) {
            stats.insert("min".to_string(), *min);
        }
        
        if let Some(max) = self.data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()) {
            stats.insert("max".to_string(), *max);
        }

        stats
    }

    pub fn filter_data<F>(&self, predicate: F) -> Vec<f64>
    where
        F: Fn(&f64) -> bool,
    {
        self.data.iter()
            .filter(|&x| predicate(x))
            .cloned()
            .collect()
    }

    pub fn get_metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn data_summary(&self) -> String {
        format!(
            "Data points: {}, Source: {}",
            self.data.len(),
            self.metadata.get("source").unwrap_or(&"unknown".to_string())
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        
        writeln!(temp_file, "value").unwrap();
        writeln!(temp_file, "10.5").unwrap();
        writeln!(temp_file, "20.3").unwrap();
        writeln!(temp_file, "15.7").unwrap();
        
        processor.load_from_csv(temp_file.path().to_str().unwrap()).unwrap();
        let stats = processor.calculate_statistics();
        
        assert_eq!(stats.get("mean").unwrap(), &15.5);
        assert_eq!(stats.get("count").unwrap(), &3.0);
        assert_eq!(stats.get("sum").unwrap(), &46.5);
    }

    #[test]
    fn test_data_filtering() {
        let mut processor = DataProcessor::new();
        processor.data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let filtered = processor.filter_data(|&x| x > 2.5);
        assert_eq!(filtered, vec![3.0, 4.0, 5.0]);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, active: bool) -> Self {
        Record {
            id,
            name,
            value,
            active,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
}

pub fn process_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid format at line {}", line_num + 1).into());
        }

        let id = parts[0].parse::<u32>()?;
        let name = parts[1].to_string();
        let value = parts[2].parse::<f64>()?;
        let active = parts[3].parse::<bool>()?;

        let record = Record::new(id, name, value, active);
        if record.is_valid() {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    if records.is_empty() {
        return (0.0, 0.0, 0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len();
    let mean = sum / count as f64;

    let variance: f64 = records
        .iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>()
        / count as f64;

    (mean, variance, count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "test".to_string(), 10.5, true);
        assert!(valid_record.is_valid());

        let invalid_record = Record::new(2, "".to_string(), -5.0, false);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_process_csv() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "1,Alice,100.5,true")?;
        writeln!(temp_file, "2,Bob,75.2,false")?;
        writeln!(temp_file, "# This is a comment")?;
        writeln!(temp_file, "")?;

        let records = process_csv_file(temp_file.path())?;
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].id, 1);
        assert_eq!(records[1].name, "Bob");

        Ok(())
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record::new(1, "A".to_string(), 10.0, true),
            Record::new(2, "B".to_string(), 20.0, true),
            Record::new(3, "C".to_string(), 30.0, false),
        ];

        let (mean, variance, count) = calculate_statistics(&records);
        assert_eq!(count, 3);
        assert!((mean - 20.0).abs() < 0.001);
        assert!((variance - 66.666).abs() < 0.001);
    }
}use csv::Reader;
use std::error::Error;
use std::fs::File;

pub struct DataProcessor {
    data: Vec<Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { data: Vec::new() }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.records() {
            let record = result?;
            let row: Vec<f64> = record.iter()
                .filter_map(|s| s.parse().ok())
                .collect();
            
            if !row.is_empty() {
                self.data.push(row);
            }
        }
        
        Ok(())
    }

    pub fn calculate_mean(&self, column_index: usize) -> Option<f64> {
        if column_index >= self.data.first()?.len() {
            return None;
        }

        let sum: f64 = self.data.iter()
            .filter_map(|row| row.get(column_index))
            .sum();
        
        let count = self.data.len() as f64;
        Some(sum / count)
    }

    pub fn calculate_standard_deviation(&self, column_index: usize) -> Option<f64> {
        let mean = self.calculate_mean(column_index)?;
        
        let variance: f64 = self.data.iter()
            .filter_map(|row| row.get(column_index))
            .map(|&value| (value - mean).powi(2))
            .sum::<f64>() / (self.data.len() as f64 - 1.0);
        
        Some(variance.sqrt())
    }

    pub fn filter_by_threshold(&self, column_index: usize, threshold: f64) -> Vec<Vec<f64>> {
        self.data.iter()
            .filter(|row| {
                row.get(column_index)
                    .map_or(false, |&value| value > threshold)
            })
            .cloned()
            .collect()
    }

    pub fn get_summary_statistics(&self) -> Vec<(usize, f64, f64)> {
        if self.data.is_empty() {
            return Vec::new();
        }

        let column_count = self.data[0].len();
        (0..column_count)
            .filter_map(|col| {
                let mean = self.calculate_mean(col)?;
                let std_dev = self.calculate_standard_deviation(col)?;
                Some((col, mean, std_dev))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1.0,2.0,3.0\n4.0,5.0,6.0\n7.0,8.0,9.0").unwrap();
        
        let mut processor = DataProcessor::new();
        processor.load_from_csv(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.calculate_mean(0), Some(4.0));
        assert_eq!(processor.calculate_standard_deviation(0), Some(3.0));
        
        let filtered = processor.filter_by_threshold(1, 4.0);
        assert_eq!(filtered.len(), 2);
        
        let stats = processor.get_summary_statistics();
        assert_eq!(stats.len(), 3);
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

    pub fn process_dataset(&mut self, dataset_name: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        let validated_data = self.validate_data(data)?;
        let transformed_data = self.apply_transformations(&validated_data);
        
        self.cache.insert(dataset_name.to_string(), transformed_data.clone());
        
        Ok(transformed_data)
    }

    fn validate_data(&self, data: &[f64]) -> Result<Vec<f64>, String> {
        for rule in &self.validation_rules {
            if rule.required && data.is_empty() {
                return Err(format!("Field '{}' is required but empty", rule.field_name));
            }
            
            for &value in data {
                if value < rule.min_value || value > rule.max_value {
                    return Err(format!(
                        "Value {} for field '{}' is outside allowed range [{}, {}]",
                        value, rule.field_name, rule.min_value, rule.max_value
                    ));
                }
            }
        }
        
        Ok(data.to_vec())
    }

    fn apply_transformations(&self, data: &[f64]) -> Vec<f64> {
        let mean = self.calculate_mean(data);
        let std_dev = self.calculate_std_dev(data, mean);
        
        data.iter()
            .map(|&x| (x - mean) / std_dev)
            .collect()
    }

    fn calculate_mean(&self, data: &[f64]) -> f64 {
        let sum: f64 = data.iter().sum();
        sum / data.len() as f64
    }

    fn calculate_std_dev(&self, data: &[f64], mean: f64) -> f64 {
        let variance: f64 = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        
        variance.sqrt()
    }

    pub fn get_cached_result(&self, dataset_name: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_name)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
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
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let rule = ValidationRule::new("temperature", -50.0, 150.0, true);
        processor.add_validation_rule(rule);

        let data = vec![20.5, 25.0, 18.3, 22.7];
        let result = processor.process_dataset("room_temps", &data);
        
        assert!(result.is_ok());
        assert_eq!(processor.get_cached_result("room_temps").unwrap().len(), 4);
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        let rule = ValidationRule::new("pressure", 0.0, 100.0, true);
        processor.add_validation_rule(rule);

        let invalid_data = vec![95.0, 105.5, 85.0];
        let result = processor.process_dataset("pressures", &invalid_data);
        
        assert!(result.is_err());
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: String,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, timestamp: String, value: f64, category: String) -> Self {
        DataRecord {
            id,
            timestamp,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.timestamp.is_empty() &&
        self.value.is_finite() &&
        !self.category.is_empty() &&
        self.category.len() <= 50
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut loaded_count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let timestamp = parts[1].to_string();
            let value = match parts[2].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let category = parts[3].to_string();

            let record = DataRecord::new(id, timestamp, value, category);
            if record.is_valid() {
                self.records.push(record);
                loaded_count += 1;
            }
        }

        Ok(loaded_count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_statistics(&self) -> (Option<f64>, Option<f64>, Option<f64>) {
        if self.records.is_empty() {
            return (None, None, None);
        }

        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average().unwrap_or(0.0);

        (Some(min), Some(max), Some(avg))
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, "2024-01-15T10:30:00Z".to_string(), 42.5, "temperature".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, "".to_string(), f64::NAN, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,2024-01-15T10:30:00Z,42.5,temperature").unwrap();
        writeln!(temp_file, "2,2024-01-15T11:00:00Z,38.2,pressure").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        assert_eq!(processor.count_records(), 2);
    }

    #[test]
    fn test_filtering() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, "2024-01-15T10:30:00Z".to_string(), 42.5, "temperature".to_string()));
        processor.records.push(DataRecord::new(2, "2024-01-15T11:00:00Z".to_string(), 38.2, "pressure".to_string()));
        processor.records.push(DataRecord::new(3, "2024-01-15T11:30:00Z".to_string(), 25.1, "temperature".to_string()));

        let temp_records = processor.filter_by_category("temperature");
        assert_eq!(temp_records.len(), 2);
        
        let pressure_records = processor.filter_by_category("pressure");
        assert_eq!(pressure_records.len(), 1);
    }

    #[test]
    fn test_statistics() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, "2024-01-15T10:30:00Z".to_string(), 10.0, "test".to_string()));
        processor.records.push(DataRecord::new(2, "2024-01-15T11:00:00Z".to_string(), 20.0, "test".to_string()));
        processor.records.push(DataRecord::new(3, "2024-01-15T11:30:00Z".to_string(), 30.0, "test".to_string()));

        let (min, max, avg) = processor.get_statistics();
        assert_eq!(min, Some(10.0));
        assert_eq!(max, Some(30.0));
        assert_eq!(avg, Some(20.0));
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    data: Vec<f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { data: Vec::new() }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            let line = line?;
            if let Ok(value) = line.trim().parse::<f64>() {
                self.data.push(value);
            }
        }
        
        Ok(())
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        
        let sum: f64 = self.data.iter().sum();
        Some(sum / self.data.len() as f64)
    }

    pub fn calculate_standard_deviation(&self) -> Option<f64> {
        if self.data.len() < 2 {
            return None;
        }
        
        let mean = self.calculate_mean()?;
        let variance: f64 = self.data
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (self.data.len() - 1) as f64;
        
        Some(variance.sqrt())
    }

    pub fn filter_outliers(&mut self, threshold: f64) {
        if let (Some(mean), Some(std_dev)) = (self.calculate_mean(), self.calculate_standard_deviation()) {
            self.data.retain(|&x| (x - mean).abs() <= threshold * std_dev);
        }
    }

    pub fn get_summary(&self) -> String {
        let mean = self.calculate_mean().unwrap_or(0.0);
        let std_dev = self.calculate_standard_deviation().unwrap_or(0.0);
        let min = self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        format!(
            "Statistics: Count={}, Mean={:.4}, StdDev={:.4}, Min={:.4}, Max={:.4}",
            self.data.len(),
            mean,
            std_dev,
            min,
            max
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "10.5\n20.3\n15.7\n25.1\n18.9").unwrap();
        
        processor.load_from_csv(temp_file.path()).unwrap();
        
        assert_eq!(processor.data.len(), 5);
        assert!(processor.calculate_mean().unwrap() - 18.1 < 0.001);
        assert!(processor.calculate_standard_deviation().unwrap() - 5.5 < 0.1);
        
        processor.filter_outliers(2.0);
        assert_eq!(processor.data.len(), 5);
        
        let summary = processor.get_summary();
        assert!(summary.contains("Count=5"));
        assert!(summary.contains("Mean=18.1"));
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
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
            DataError::InvalidId => write!(f, "Record ID must be greater than zero"),
            DataError::EmptyValues => write!(f, "Record must contain at least one value"),
            DataError::ValueOutOfRange(val) => write!(f, "Value {} is out of acceptable range", val),
            DataError::MissingMetadata(key) => write!(f, "Required metadata '{}' is missing", key),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    min_value: f64,
    max_value: f64,
    required_metadata: Vec<String>,
}

impl DataProcessor {
    pub fn new(min_value: f64, max_value: f64, required_metadata: Vec<String>) -> Self {
        DataProcessor {
            min_value,
            max_value,
            required_metadata,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }

        if record.values.is_empty() {
            return Err(DataError::EmptyValues);
        }

        for &value in &record.values {
            if value < self.min_value || value > self.max_value {
                return Err(DataError::ValueOutOfRange(value));
            }
        }

        for key in &self.required_metadata {
            if !record.metadata.contains_key(key) {
                return Err(DataError::MissingMetadata(key.clone()));
            }
        }

        Ok(())
    }

    pub fn normalize_values(&self, record: &mut DataRecord) {
        let min_val = record.values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_val = record.values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if max_val > min_val {
            for value in &mut record.values {
                *value = (*value - min_val) / (max_val - min_val);
            }
        }
    }

    pub fn process_records(&self, records: &mut [DataRecord]) -> Vec<Result<DataRecord, DataError>> {
        let mut results = Vec::new();

        for record in records {
            match self.validate_record(record) {
                Ok(_) => {
                    let mut processed_record = record.clone();
                    self.normalize_values(&mut processed_record);
                    results.push(Ok(processed_record));
                }
                Err(e) => {
                    results.push(Err(e));
                }
            }
        }

        results
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if records.is_empty() {
            return stats;
        }

        let value_count = records.iter().map(|r| r.values.len()).sum::<usize>();
        let all_values: Vec<f64> = records.iter().flat_map(|r| r.values.clone()).collect();
        
        let sum: f64 = all_values.iter().sum();
        let mean = sum / (value_count as f64);
        
        let variance: f64 = all_values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (value_count as f64);
        
        stats.insert("record_count".to_string(), records.len() as f64);
        stats.insert("value_count".to_string(), value_count as f64);
        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("min".to_string(), all_values.iter().fold(f64::INFINITY, |a, &b| a.min(b)));
        stats.insert("max".to_string(), all_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)));

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record() -> DataRecord {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        metadata.insert("version".to_string(), "1.0".to_string());

        DataRecord {
            id: 1,
            values: vec![10.0, 20.0, 30.0],
            metadata,
        }
    }

    #[test]
    fn test_valid_record() {
        let processor = DataProcessor::new(0.0, 100.0, vec!["source".to_string()]);
        let record = create_test_record();
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let processor = DataProcessor::new(0.0, 100.0, vec![]);
        let mut record = create_test_record();
        record.id = 0;
        
        assert!(matches!(processor.validate_record(&record), Err(DataError::InvalidId)));
    }

    #[test]
    fn test_normalization() {
        let processor = DataProcessor::new(0.0, 100.0, vec![]);
        let mut record = create_test_record();
        
        processor.normalize_values(&mut record);
        
        assert_eq!(record.values[0], 0.0);
        assert_eq!(record.values[1], 0.5);
        assert_eq!(record.values[2], 1.0);
    }

    #[test]
    fn test_statistics() {
        let processor = DataProcessor::new(0.0, 100.0, vec![]);
        let records = vec![create_test_record()];
        
        let stats = processor.calculate_statistics(&records);
        
        assert_eq!(stats.get("record_count"), Some(&1.0));
        assert_eq!(stats.get("value_count"), Some(&3.0));
        assert_eq!(stats.get("mean"), Some(&20.0));
    }
}