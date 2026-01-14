
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    values: Vec<f64>,
    metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    EmptyValues,
    ValueOutOfRange(f64),
    MissingMetadata(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::EmptyValues => write!(f, "Record contains no values"),
            DataError::ValueOutOfRange(val) => write!(f, "Value {} is out of acceptable range", val),
            DataError::MissingMetadata(key) => write!(f, "Missing metadata key: {}", key),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>, metadata: HashMap<String, String>) -> Self {
        DataRecord {
            id,
            values,
            metadata,
        }
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::InvalidId);
        }

        if self.values.is_empty() {
            return Err(DataError::EmptyValues);
        }

        for &value in &self.values {
            if !value.is_finite() || value < 0.0 || value > 1000.0 {
                return Err(DataError::ValueOutOfRange(value));
            }
        }

        if !self.metadata.contains_key("source") {
            return Err(DataError::MissingMetadata("source".to_string()));
        }

        Ok(())
    }

    pub fn transform(&mut self, multiplier: f64) -> &mut Self {
        for value in &mut self.values {
            *value *= multiplier;
        }
        self
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        let sum: f64 = self.values.iter().sum();
        let count = self.values.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }
}

pub fn process_records(records: &mut [DataRecord], multiplier: f64) -> Result<Vec<(u32, f64, f64, f64)>, DataError> {
    let mut results = Vec::new();

    for record in records {
        record.validate()?;
        record.transform(multiplier);
        
        let (mean, variance, std_dev) = record.calculate_statistics();
        results.push((record.id, mean, variance, std_dev));
    }

    Ok(results)
}

pub fn filter_records_by_threshold(records: &[DataRecord], threshold: f64) -> Vec<&DataRecord> {
    records
        .iter()
        .filter(|record| {
            let (mean, _, _) = record.calculate_statistics();
            mean >= threshold
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record() -> DataRecord {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        metadata.insert("timestamp".to_string(), "2024-01-01".to_string());

        DataRecord::new(
            1,
            vec![10.0, 20.0, 30.0, 40.0, 50.0],
            metadata,
        )
    }

    #[test]
    fn test_record_validation() {
        let record = create_test_record();
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_record_transformation() {
        let mut record = create_test_record();
        record.transform(2.0);
        
        let expected_values = vec![20.0, 40.0, 60.0, 80.0, 100.0];
        assert_eq!(record.values, expected_values);
    }

    #[test]
    fn test_statistics_calculation() {
        let record = create_test_record();
        let (mean, variance, std_dev) = record.calculate_statistics();
        
        assert_eq!(mean, 30.0);
        assert_eq!(variance, 200.0);
        assert_eq!(std_dev, 200.0_f64.sqrt());
    }

    #[test]
    fn test_process_records() {
        let mut records = vec![create_test_record()];
        let result = process_records(&mut records, 1.0);
        
        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.len(), 1);
    }

    #[test]
    fn test_filter_records() {
        let records = vec![create_test_record()];
        let filtered = filter_records_by_threshold(&records, 25.0);
        
        assert_eq!(filtered.len(), 1);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(Self { id, value, category })
    }

    pub fn calculate_adjusted_value(&self, multiplier: f64) -> f64 {
        self.value * multiplier
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self { records: Vec::new() }
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

            let id = parts[0].parse::<u32>()?;
            let value = parts[1].parse::<f64>()?;
            let category = parts[2].to_string();

            match DataRecord::new(id, value, category) {
                Ok(record) => {
                    self.records.push(record);
                    count += 1;
                }
                Err(e) => eprintln!("Skipping invalid record at line {}: {}", line_num + 1, e),
            }
        }

        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        let count = self.records.len() as f64;
        if count == 0.0 {
            return (0.0, 0.0, 0.0);
        }

        let total: f64 = self.records.iter().map(|r| r.value).sum();
        let mean = total / count;

        let variance: f64 = self.records
            .iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }

    pub fn export_summary(&self) -> String {
        let (mean, variance, std_dev) = self.get_statistics();
        let total = self.calculate_total_value();
        let categories: std::collections::HashSet<_> = self.records.iter().map(|r| &r.category).collect();

        format!(
            "Total records: {}\nTotal value: {:.2}\nMean: {:.2}\nVariance: {:.2}\nStd Dev: {:.2}\nCategories: {}",
            self.records.len(),
            total,
            mean,
            variance,
            std_dev,
            categories.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, 42.5, "test".to_string()).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
    }

    #[test]
    fn test_invalid_record() {
        let result = DataRecord::new(1, -5.0, "test".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,category_a").unwrap();
        writeln!(temp_file, "2,20.0,category_b").unwrap();
        writeln!(temp_file, "3,15.5,category_a").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.calculate_total_value(), 46.0);
        
        let category_a = processor.filter_by_category("category_a");
        assert_eq!(category_a.len(), 2);
    }
}