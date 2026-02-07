use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::path::Path;

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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let mut rdr = Reader::from_path(path)?;
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "id,name,value,category\n1,ItemA,25.5,Alpha\n2,ItemB,-10.0,Beta\n3,,15.0,Alpha"
        )
        .unwrap();

        let mut processor = DataProcessor::new();
        processor.load_from_csv(file.path()).unwrap();

        assert_eq!(processor.records.len(), 3);
        assert_eq!(processor.validate_records().len(), 1);
        assert_eq!(processor.calculate_total(), 30.5);

        let grouped = processor.group_by_category();
        assert_eq!(grouped.get("Alpha").unwrap().len(), 2);
        assert_eq!(grouped.get("Beta").unwrap().len(), 1);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    InvalidValue,
    InvalidTimestamp,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "Invalid record ID"),
            ValidationError::InvalidValue => write!(f, "Invalid value field"),
            ValidationError::InvalidTimestamp => write!(f, "Invalid timestamp"),
        }
    }
}

impl Error for ValidationError {}

pub fn validate_record(record: &DataRecord) -> Result<(), ValidationError> {
    if record.id == 0 {
        return Err(ValidationError::InvalidId);
    }
    
    if !record.value.is_finite() {
        return Err(ValidationError::InvalidValue);
    }
    
    if record.timestamp < 0 {
        return Err(ValidationError::InvalidTimestamp);
    }
    
    Ok(())
}

pub fn transform_record(record: &DataRecord) -> DataRecord {
    DataRecord {
        id: record.id,
        value: record.value * 1.1,
        timestamp: record.timestamp + 3600,
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Vec<Result<DataRecord, ValidationError>> {
    records
        .into_iter()
        .map(|record| {
            validate_record(&record)?;
            Ok(transform_record(&record))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            value: 42.5,
            timestamp: 1672531200,
        };
        
        assert!(validate_record(&record).is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            value: 42.5,
            timestamp: 1672531200,
        };
        
        assert!(matches!(validate_record(&record), Err(ValidationError::InvalidId)));
    }

    #[test]
    fn test_transform_record() {
        let original = DataRecord {
            id: 1,
            value: 100.0,
            timestamp: 1000,
        };
        
        let transformed = transform_record(&original);
        
        assert_eq!(transformed.id, 1);
        assert_eq!(transformed.value, 110.0);
        assert_eq!(transformed.timestamp, 4600);
    }
}use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidValue,
    EmptyName,
    DuplicateTag,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than 0"),
            DataError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            DataError::EmptyName => write!(f, "Name cannot be empty"),
            DataError::DuplicateTag => write!(f, "Duplicate tag found"),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, tags: Vec<String>) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        if name.trim().is_empty() {
            return Err(DataError::EmptyName);
        }
        if value < 0.0 || value > 1000.0 {
            return Err(DataError::InvalidValue);
        }
        
        let mut seen_tags = HashMap::new();
        for tag in &tags {
            if seen_tags.contains_key(tag) {
                return Err(DataError::DuplicateTag);
            }
            seen_tags.insert(tag.clone(), true);
        }

        Ok(Self {
            id,
            name,
            value,
            tags,
        })
    }

    pub fn transform_value(&mut self, multiplier: f64) {
        self.value *= multiplier;
        if self.value > 1000.0 {
            self.value = 1000.0;
        }
    }

    pub fn add_tag(&mut self, tag: String) -> Result<(), DataError> {
        if self.tags.contains(&tag) {
            return Err(DataError::DuplicateTag);
        }
        self.tags.push(tag);
        Ok(())
    }

    pub fn calculate_score(&self) -> f64 {
        let base_score = self.value / 10.0;
        let tag_bonus = self.tags.len() as f64 * 2.5;
        base_score + tag_bonus
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Vec<f64> {
    records
        .iter_mut()
        .map(|record| {
            record.transform_value(1.1);
            record.calculate_score()
        })
        .collect()
}

pub fn filter_records_by_threshold(records: &[DataRecord], threshold: f64) -> Vec<&DataRecord> {
    records
        .iter()
        .filter(|record| record.value >= threshold)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(
            1,
            "Test Record".to_string(),
            500.0,
            vec!["tag1".to_string(), "tag2".to_string()],
        );
        assert!(record.is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(
            0,
            "Test".to_string(),
            500.0,
            vec![],
        );
        assert!(matches!(record, Err(DataError::InvalidId)));
    }

    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord::new(
            1,
            "Test".to_string(),
            500.0,
            vec![],
        ).unwrap();
        record.transform_value(1.5);
        assert_eq!(record.value, 750.0);
    }

    #[test]
    fn test_calculate_score() {
        let record = DataRecord::new(
            1,
            "Test".to_string(),
            200.0,
            vec!["a".to_string(), "b".to_string()],
        ).unwrap();
        let score = record.calculate_score();
        assert_eq!(score, 25.0); // 200/10 + 2*2.5 = 20 + 5 = 25
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
    valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        let valid = value >= 0.0 && !category.is_empty();
        Self {
            id,
            value,
            category,
            valid,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn get_value(&self) -> f64 {
        self.value
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    total_value: f64,
    valid_count: usize,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            total_value: 0.0,
            valid_count: 0,
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }

            let id = parts[0].parse().unwrap_or(0);
            let value = parts[1].parse().unwrap_or(0.0);
            let category = parts[2].to_string();

            let record = DataRecord::new(id, value, category);
            self.add_record(record);
        }

        Ok(())
    }

    pub fn add_record(&mut self, record: DataRecord) {
        if record.is_valid() {
            self.total_value += record.get_value();
            self.valid_count += 1;
        }
        self.records.push(record);
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.valid_count > 0 {
            Some(self.total_value / self.valid_count as f64)
        } else {
            None
        }
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category && r.is_valid())
            .collect()
    }

    pub fn get_statistics(&self) -> (usize, usize, Option<f64>) {
        let total = self.records.len();
        let average = self.calculate_average();
        (total, self.valid_count, average)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, 10.5, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_value = DataRecord::new(2, -5.0, "B".to_string());
        assert!(!invalid_value.is_valid());

        let invalid_category = DataRecord::new(3, 15.0, "".to_string());
        assert!(!invalid_category.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        processor.add_record(DataRecord::new(1, 10.0, "X".to_string()));
        processor.add_record(DataRecord::new(2, 20.0, "X".to_string()));
        processor.add_record(DataRecord::new(3, -5.0, "Y".to_string()));

        let (total, valid, avg) = processor.get_statistics();
        assert_eq!(total, 3);
        assert_eq!(valid, 2);
        assert_eq!(avg, Some(15.0));

        let filtered = processor.filter_by_category("X");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_csv_loading() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "id,value,category")?;
        writeln!(temp_file, "1,10.5,TypeA")?;
        writeln!(temp_file, "2,20.3,TypeB")?;
        writeln!(temp_file, "3,-5.0,TypeA")?;

        let mut processor = DataProcessor::new();
        processor.load_from_csv(temp_file.path())?;

        let (total, valid, avg) = processor.get_statistics();
        assert_eq!(total, 3);
        assert_eq!(valid, 2);
        assert!(avg.is_some());

        Ok(())
    }
}