
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: HashMap::new(),
        }
    }

    pub fn add_dataset(&mut self, key: String, values: Vec<f64>) -> Result<(), String> {
        if values.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        if values.iter().any(|&x| x.is_nan() || x.is_infinite()) {
            return Err("Dataset contains invalid numeric values".to_string());
        }

        self.data.insert(key, values);
        Ok(())
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<Statistics> {
        self.data.get(key).map(|values| {
            let count = values.len();
            let sum: f64 = values.iter().sum();
            let mean = sum / count as f64;
            
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count as f64;
            
            let std_dev = variance.sqrt();

            Statistics {
                count,
                mean,
                variance,
                std_dev,
                min: *values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
                max: *values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
            }
        })
    }

    pub fn normalize_data(&self, key: &str) -> Option<Vec<f64>> {
        self.calculate_statistics(key).map(|stats| {
            self.data[key].iter()
                .map(|&x| (x - stats.min) / (stats.max - stats.min))
                .collect()
        })
    }

    pub fn get_keys(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }
}

pub struct Statistics {
    pub count: usize,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.add_dataset("test".to_string(), vec![1.0, 2.0, 3.0]);
        assert!(result.is_ok());
        assert_eq!(processor.get_keys(), vec!["test".to_string()]);
    }

    #[test]
    fn test_add_invalid_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.add_dataset("invalid".to_string(), vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_statistics() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("numbers".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        
        let stats = processor.calculate_statistics("numbers").unwrap();
        assert_eq!(stats.count, 5);
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
    }

    #[test]
    fn test_normalize_data() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("values".to_string(), vec![10.0, 20.0, 30.0]).unwrap();
        
        let normalized = processor.normalize_data("values").unwrap();
        assert_eq!(normalized, vec![0.0, 0.5, 1.0]);
    }
}use std::error::Error;
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

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String]) -> bool {
        !record.is_empty() && record.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Option<(f64, f64)> {
        let mut values = Vec::new();
        
        for record in records {
            if column_index < record.len() {
                if let Ok(value) = record[column_index].parse::<f64>() {
                    values.push(value);
                }
            }
        }

        if values.is_empty() {
            return None;
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;
        
        let variance: f64 = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        
        Some((mean, variance.sqrt()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,score").unwrap();
        writeln!(temp_file, "Alice,25,95.5").unwrap();
        writeln!(temp_file, "Bob,30,88.0").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "25", "95.5"]);
    }

    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(',', false);
        let valid_record = vec!["data".to_string(), "123".to_string()];
        let invalid_record = vec!["".to_string(), "test".to_string()];
        
        assert!(processor.validate_record(&valid_record));
        assert!(!processor.validate_record(&invalid_record));
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(',', false);
        let records = vec![
            vec!["10.0".to_string(), "20.0".to_string()],
            vec!["20.0".to_string(), "30.0".to_string()],
            vec!["30.0".to_string(), "40.0".to_string()],
        ];
        
        let stats = processor.calculate_statistics(&records, 0).unwrap();
        assert!((stats.0 - 20.0).abs() < 0.001);
        assert!((stats.1 - 8.1649).abs() < 0.001);
    }
}
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let count = records.len();
    if count == 0 {
        return (0.0, 0.0, 0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let average = sum / count as f64;
    let max_value = records.iter().map(|r| r.value).fold(0.0, f64::max);

    (average, max_value, count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_valid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,20.3,Category2").unwrap();

        let records = process_data_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "ItemA");
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record { id: 1, name: "Test1".to_string(), value: 10.0, category: "Cat1".to_string() },
            Record { id: 2, name: "Test2".to_string(), value: 20.0, category: "Cat2".to_string() },
        ];

        let (avg, max, count) = calculate_statistics(&records);
        assert_eq!(avg, 15.0);
        assert_eq!(max, 20.0);
        assert_eq!(count, 2);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataPoint {
    timestamp: i64,
    value: f64,
    category: String,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidTimestamp,
    InvalidValue,
    EmptyCategory,
    TransformationFailed,
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidTimestamp => write!(f, "Timestamp must be positive"),
            ProcessingError::InvalidValue => write!(f, "Value must be finite"),
            ProcessingError::EmptyCategory => write!(f, "Category cannot be empty"),
            ProcessingError::TransformationFailed => write!(f, "Data transformation failed"),
        }
    }
}

impl Error for ProcessingError {}

impl DataPoint {
    pub fn new(timestamp: i64, value: f64, category: String) -> Result<Self, ProcessingError> {
        if timestamp <= 0 {
            return Err(ProcessingError::InvalidTimestamp);
        }
        
        if !value.is_finite() {
            return Err(ProcessingError::InvalidValue);
        }
        
        if category.trim().is_empty() {
            return Err(ProcessingError::EmptyCategory);
        }
        
        Ok(Self {
            timestamp,
            value,
            category,
        })
    }
    
    pub fn transform(&self, multiplier: f64) -> Result<Self, ProcessingError> {
        if !multiplier.is_finite() || multiplier == 0.0 {
            return Err(ProcessingError::TransformationFailed);
        }
        
        let transformed_value = self.value * multiplier;
        
        DataPoint::new(
            self.timestamp,
            transformed_value,
            self.category.clone(),
        )
    }
    
    pub fn normalize(&self, max_value: f64) -> Result<Self, ProcessingError> {
        if max_value <= 0.0 || !max_value.is_finite() {
            return Err(ProcessingError::TransformationFailed);
        }
        
        let normalized_value = self.value / max_value;
        
        DataPoint::new(
            self.timestamp,
            normalized_value,
            self.category.clone(),
        )
    }
    
    pub fn get_timestamp(&self) -> i64 {
        self.timestamp
    }
    
    pub fn get_value(&self) -> f64 {
        self.value
    }
    
    pub fn get_category(&self) -> &str {
        &self.category
    }
}

pub struct DataProcessor {
    points: Vec<DataPoint>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
        }
    }
    
    pub fn add_point(&mut self, point: DataPoint) {
        self.points.push(point);
    }
    
    pub fn process_all(&mut self, operation: fn(&DataPoint) -> Result<DataPoint, ProcessingError>) 
        -> Result<(), ProcessingError> {
        let mut processed_points = Vec::new();
        
        for point in &self.points {
            let processed = operation(point)?;
            processed_points.push(processed);
        }
        
        self.points = processed_points;
        Ok(())
    }
    
    pub fn filter_by_category(&self, category: &str) -> Vec<&DataPoint> {
        self.points
            .iter()
            .filter(|p| p.get_category() == category)
            .collect()
    }
    
    pub fn calculate_average(&self) -> Option<f64> {
        if self.points.is_empty() {
            return None;
        }
        
        let sum: f64 = self.points.iter().map(|p| p.get_value()).sum();
        Some(sum / self.points.len() as f64)
    }
    
    pub fn get_points(&self) -> &[DataPoint] {
        &self.points
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_data_point() {
        let point = DataPoint::new(1234567890, 42.5, "temperature".to_string());
        assert!(point.is_ok());
    }
    
    #[test]
    fn test_invalid_timestamp() {
        let point = DataPoint::new(-1, 42.5, "temperature".to_string());
        assert!(matches!(point, Err(ProcessingError::InvalidTimestamp)));
    }
    
    #[test]
    fn test_data_transformation() {
        let point = DataPoint::new(1234567890, 10.0, "pressure".to_string()).unwrap();
        let transformed = point.transform(2.5).unwrap();
        assert_eq!(transformed.get_value(), 25.0);
    }
}