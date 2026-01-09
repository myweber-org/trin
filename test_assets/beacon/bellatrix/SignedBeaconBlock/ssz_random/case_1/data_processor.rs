
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Invalid data value: {0}")]
    InvalidValue(f64),
    #[error("Timestamp out of range: {0}")]
    InvalidTimestamp(i64),
    #[error("Duplicate record ID: {0}")]
    DuplicateId(u32),
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    processed_ids: std::collections::HashSet<u32>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            processed_ids: std::collections::HashSet::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        if record.value.is_nan() || record.value.is_infinite() {
            return Err(DataError::InvalidValue(record.value));
        }

        if record.timestamp < 0 || record.timestamp > 253402300799 {
            return Err(DataError::InvalidTimestamp(record.timestamp));
        }

        if self.processed_ids.contains(&record.id) {
            return Err(DataError::DuplicateId(record.id));
        }

        self.processed_ids.insert(record.id);
        self.records.push(record);
        Ok(())
    }

    pub fn calculate_statistics(&self) -> Option<Statistics> {
        if self.records.is_empty() {
            return None;
        }

        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let count = values.len();
        let sum: f64 = values.iter().sum();
        let mean = sum / count as f64;
        let variance: f64 = values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        Some(Statistics {
            count,
            mean,
            std_dev,
            min,
            max,
            sum,
        })
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= threshold)
            .collect()
    }

    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.processed_ids.clear();
    }
}

#[derive(Debug, Clone)]
pub struct Statistics {
    pub count: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub sum: f64,
}

impl std::fmt::Display for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Statistics: count={}, mean={:.2}, std_dev={:.2}, min={:.2}, max={:.2}, sum={:.2}",
            self.count, self.mean, self.std_dev, self.min, self.max, self.sum
        )
    }
}
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataSet {
    values: Vec<f64>,
}

impl DataSet {
    pub fn new() -> Self {
        DataSet { values: Vec::new() }
    }

    pub fn from_csv<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        let mut values = Vec::new();

        for result in rdr.records() {
            let record = result?;
            if let Some(field) = record.get(0) {
                if let Ok(value) = field.parse::<f64>() {
                    values.push(value);
                }
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
        let mean = self.mean()?;
        let sum_sq_diff: f64 = self.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum();
        Some(sum_sq_diff / (self.values.len() - 1) as f64)
    }

    pub fn standard_deviation(&self) -> Option<f64> {
        self.variance().map(|v| v.sqrt())
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
    fn test_empty_dataset() {
        let ds = DataSet::new();
        assert_eq!(ds.mean(), None);
        assert_eq!(ds.count(), 0);
    }

    #[test]
    fn test_basic_statistics() {
        let mut ds = DataSet::new();
        ds.add_value(10.0);
        ds.add_value(20.0);
        ds.add_value(30.0);
        
        assert_eq!(ds.mean(), Some(20.0));
        assert_eq!(ds.variance(), Some(100.0));
        assert_eq!(ds.standard_deviation(), Some(10.0));
        assert_eq!(ds.min(), Some(10.0));
        assert_eq!(ds.max(), Some(30.0));
        assert_eq!(ds.count(), 3);
    }

    #[test]
    fn test_csv_parsing() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "value")?;
        writeln!(temp_file, "5.5")?;
        writeln!(temp_file, "6.5")?;
        writeln!(temp_file, "7.5")?;
        
        let ds = DataSet::from_csv(temp_file.path())?;
        assert_eq!(ds.mean(), Some(6.5));
        assert_eq!(ds.count(), 3);
        Ok(())
    }
}
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, timestamp: i64) -> Self {
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

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.id == 0 {
            return Err("Invalid record ID".into());
        }
        if self.timestamp < 0 {
            return Err("Invalid timestamp".into());
        }
        if self.values.is_empty() {
            return Err("No values provided".into());
        }
        Ok(())
    }
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    let total_records = records.len() as f64;

    if total_records == 0.0 {
        return stats;
    }

    let mut sum_values = 0.0;
    let mut count_values = 0;

    for record in records {
        for &value in &record.values {
            sum_values += value;
            count_values += 1;
        }
    }

    if count_values > 0 {
        let avg_value = sum_values / count_values as f64;
        stats.insert("average".to_string(), avg_value);
        stats.insert("total_records".to_string(), total_records);
        stats.insert("total_values".to_string(), count_values as f64);
    }

    stats
}

pub fn transform_records(records: &[DataRecord], multiplier: f64) -> Vec<DataRecord> {
    records
        .iter()
        .map(|record| {
            let mut transformed = record.clone();
            transformed.values = record.values.iter().map(|&v| v * multiplier).collect();
            transformed
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, 1625097600);
        assert_eq!(record.id, 1);
        assert_eq!(record.timestamp, 1625097600);
        assert!(record.values.is_empty());
    }

    #[test]
    fn test_record_validation() {
        let mut valid_record = DataRecord::new(1, 1625097600);
        valid_record.add_value(42.0);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, 1625097600);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut record1 = DataRecord::new(1, 1625097600);
        record1.add_value(10.0).add_value(20.0);

        let mut record2 = DataRecord::new(2, 1625097601);
        record2.add_value(30.0);

        let records = vec![record1, record2];
        let stats = calculate_statistics(&records);

        assert_eq!(stats.get("average"), Some(&20.0));
        assert_eq!(stats.get("total_records"), Some(&2.0));
        assert_eq!(stats.get("total_values"), Some(&3.0));
    }

    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord::new(1, 1625097600);
        record.add_value(10.0).add_value(20.0);

        let transformed = transform_records(&[record], 2.0);
        assert_eq!(transformed[0].values, vec![20.0, 40.0]);
    }
}