use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataSet {
    values: Vec<f64>,
}

impl DataSet {
    pub fn new() -> Self {
        DataSet { values: Vec::new() }
    }

    pub fn from_csv(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut values = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(num) = line.trim().parse::<f64>() {
                values.push(num);
            }
        }

        Ok(DataSet { values })
    }

    pub fn add_value(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn mean(&self) -> Option<f64> {
        if self.values.is_empty() {
            return None;
        }
        let sum: f64 = self.values.iter().sum();
        Some(sum / self.values.len() as f64)
    }

    pub fn variance(&self) -> Option<f64> {
        if self.values.len() < 2 {
            return None;
        }
        let mean = self.mean().unwrap();
        let sum_sq_diff: f64 = self.values.iter().map(|&x| (x - mean).powi(2)).sum();
        Some(sum_sq_diff / (self.values.len() - 1) as f64)
    }

    pub fn min(&self) -> Option<f64> {
        self.values.iter().copied().reduce(f64::min)
    }

    pub fn max(&self) -> Option<f64> {
        self.values.iter().copied().reduce(f64::max)
    }

    pub fn count(&self) -> usize {
        self.values.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_statistics() {
        let mut dataset = DataSet::new();
        dataset.add_value(10.0);
        dataset.add_value(20.0);
        dataset.add_value(30.0);

        assert_eq!(dataset.mean(), Some(20.0));
        assert_eq!(dataset.variance(), Some(100.0));
        assert_eq!(dataset.min(), Some(10.0));
        assert_eq!(dataset.max(), Some(30.0));
        assert_eq!(dataset.count(), 3);
    }

    #[test]
    fn test_empty_dataset() {
        let dataset = DataSet::new();
        assert_eq!(dataset.mean(), None);
        assert_eq!(dataset.variance(), None);
        assert_eq!(dataset.min(), None);
        assert_eq!(dataset.max(), None);
        assert_eq!(dataset.count(), 0);
    }

    #[test]
    fn test_csv_import() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "5.5\n10.2\n15.7\n").unwrap();
        
        let dataset = DataSet::from_csv(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(dataset.mean(), Some(10.466666666666667));
        assert_eq!(dataset.count(), 3);
    }
}
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

    pub fn process_file<P: AsRef<Path>>(
        &self,
        file_path: P,
        filter_predicate: Option<fn(&[String]) -> bool>,
    ) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if self.has_header {
            let _ = lines.next();
        }

        let mut records = Vec::new();

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if let Some(predicate) = filter_predicate {
                if predicate(&fields) {
                    records.push(fields);
                }
            } else {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn calculate_statistics(&self, data: &[Vec<String>], column_index: usize) -> Option<f64> {
        if data.is_empty() {
            return None;
        }

        let mut sum = 0.0;
        let mut count = 0;

        for record in data {
            if column_index < record.len() {
                if let Ok(value) = record[column_index].parse::<f64>() {
                    sum += value;
                    count += 1;
                }
            }
        }

        if count > 0 {
            Some(sum / count as f64)
        } else {
            None
        }
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
        writeln!(temp_file, "Charlie,35,Paris").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path(), None).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_calculate_statistics() {
        let data = vec![
            vec!["10.5".to_string(), "20.0".to_string()],
            vec!["15.5".to_string(), "30.0".to_string()],
            vec!["12.0".to_string(), "25.0".to_string()],
        ];

        let processor = DataProcessor::new(',', false);
        let avg = processor.calculate_statistics(&data, 0).unwrap();

        assert!((avg - 12.666).abs() < 0.001);
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
pub enum ProcessingError {
    InvalidData(String),
    TransformationFailed(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    category_stats: HashMap<String, CategoryStatistics>,
}

#[derive(Debug, Clone)]
pub struct CategoryStatistics {
    pub total_value: f64,
    pub record_count: usize,
    pub average_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            category_stats: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ProcessingError> {
        self.validate_record(&record)?;
        self.records.push(record.clone());
        self.update_category_stats(&record);
        Ok(())
    }

    pub fn process_records(&mut self) -> Result<(), ProcessingError> {
        if self.records.is_empty() {
            return Err(ProcessingError::InvalidData("No records to process".to_string()));
        }

        self.calculate_statistics();
        self.normalize_values()?;
        Ok(())
    }

    pub fn get_category_statistics(&self, category: &str) -> Option<&CategoryStatistics> {
        self.category_stats.get(category)
    }

    pub fn get_all_statistics(&self) -> &HashMap<String, CategoryStatistics> {
        &self.category_stats
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= threshold)
            .collect()
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.name.trim().is_empty() {
            return Err(ProcessingError::ValidationError("Record name cannot be empty".to_string()));
        }
        
        if record.value < 0.0 {
            return Err(ProcessingError::ValidationError("Record value cannot be negative".to_string()));
        }
        
        if record.category.trim().is_empty() {
            return Err(ProcessingError::ValidationError("Category cannot be empty".to_string()));
        }
        
        Ok(())
    }

    fn update_category_stats(&mut self, record: &DataRecord) {
        let stats = self.category_stats
            .entry(record.category.clone())
            .or_insert(CategoryStatistics {
                total_value: 0.0,
                record_count: 0,
                average_value: 0.0,
            });
        
        stats.total_value += record.value;
        stats.record_count += 1;
        stats.average_value = stats.total_value / stats.record_count as f64;
    }

    fn calculate_statistics(&mut self) {
        self.category_stats.clear();
        for record in &self.records {
            self.update_category_stats(record);
        }
    }

    fn normalize_values(&mut self) -> Result<(), ProcessingError> {
        if self.records.is_empty() {
            return Ok(());
        }

        let max_value = self.records
            .iter()
            .map(|r| r.value)
            .fold(f64::NEG_INFINITY, f64::max);

        if max_value <= 0.0 {
            return Err(ProcessingError::TransformationFailed(
                "Cannot normalize: maximum value is not positive".to_string()
            ));
        }

        for record in &mut self.records {
            record.value = record.value / max_value;
        }

        self.calculate_statistics();
        Ok(())
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
            category: "CategoryA".to_string(),
        };

        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.records.len(), 1);
    }

    #[test]
    fn test_add_invalid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "".to_string(),
            value: 100.0,
            category: "CategoryA".to_string(),
        };

        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_category_statistics() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord {
            id: 1,
            name: "Record 1".to_string(),
            value: 50.0,
            category: "CategoryA".to_string(),
        };

        let record2 = DataRecord {
            id: 2,
            name: "Record 2".to_string(),
            value: 150.0,
            category: "CategoryA".to_string(),
        };

        processor.add_record(record1).unwrap();
        processor.add_record(record2).unwrap();
        processor.process_records().unwrap();

        let stats = processor.get_category_statistics("CategoryA").unwrap();
        assert_eq!(stats.total_value, 200.0);
        assert_eq!(stats.record_count, 2);
        assert_eq!(stats.average_value, 100.0);
    }

    #[test]
    fn test_filter_records() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord { id: 1, name: "Low".to_string(), value: 10.0, category: "Test".to_string() },
            DataRecord { id: 2, name: "Medium".to_string(), value: 50.0, category: "Test".to_string() },
            DataRecord { id: 3, name: "High".to_string(), value: 100.0, category: "Test".to_string() },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        let filtered = processor.filter_by_threshold(50.0);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.value >= 50.0));
    }
}use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
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

    fn load_from_csv(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    fn save_filtered_to_csv(&self, category: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        let file = File::create(output_path)?;
        let mut wtr = WriterBuilder::new().has_headers(true).from_writer(file);
        
        for record in filtered {
            wtr.serialize(record)?;
        }
        
        wtr.flush()?;
        Ok(())
    }

    fn sort_by_value(&mut self) {
        self.records.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap());
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.load_from_csv("input_data.csv")?;
    
    println!("Total records loaded: {}", processor.records.len());
    println!("Average value: {:.2}", processor.calculate_average());
    
    let filtered = processor.filter_by_category("premium");
    println!("Premium records: {}", filtered.len());
    
    processor.sort_by_value();
    processor.save_filtered_to_csv("premium", "premium_data.csv")?;
    
    Ok(())
}use std::error::Error;
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

    pub fn validate(&self) -> bool {
        self.valid && self.id > 0
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

            let category = parts[2].trim();
            let record = DataRecord::new(id, value, category);
            self.records.push(record);
            count += 1;
        }

        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.validate())
            .collect()
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
            if record.validate() {
                groups
                    .entry(record.category.clone())
                    .or_insert_with(Vec::new)
                    .push(record);
            }
        }
        
        groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 10.5, "A");
        assert!(valid_record.validate());

        let invalid_record = DataRecord::new(0, -5.0, "");
        assert!(!invalid_record.validate());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,TypeA").unwrap();
        writeln!(temp_file, "2,15.3,TypeB").unwrap();
        writeln!(temp_file, "3,-5.0,TypeA").unwrap();
        
        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        
        let valid_count = processor.filter_valid().len();
        assert_eq!(valid_count, 2);
        
        let avg = processor.calculate_average().unwrap();
        assert!((avg - 12.9).abs() < 0.001);
        
        let groups = processor.group_by_category();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("TypeA").unwrap().len(), 1);
        assert_eq!(groups.get("TypeB").unwrap().len(), 1);
    }
}