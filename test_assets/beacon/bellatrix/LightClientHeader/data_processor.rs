
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

pub struct DataProcessor {
    records: Vec<Record>,
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
            if parts.len() != 4 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let name = parts[1].to_string();
            
            let value = match parts[2].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let active = match parts[3].to_lowercase().as_str() {
                "true" | "1" | "yes" => true,
                _ => false,
            };

            let record = Record::new(id, name, value, active);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_by_value(&self, threshold: f64) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.value > threshold)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_active_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "Test".to_string(), 10.5, true);
        assert!(valid_record.is_valid());

        let invalid_record = Record::new(2, "".to_string(), -5.0, false);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.count_records(), 0);
        
        let record = Record::new(1, "Sample".to_string(), 100.0, true);
        processor.records.push(record);
        
        assert_eq!(processor.count_records(), 1);
        assert_eq!(processor.calculate_average(), Some(100.0));
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
pub enum DataError {
    InvalidTimestamp,
    InvalidValue,
    EmptyCategory,
    TransformationError(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidTimestamp => write!(f, "Timestamp must be positive"),
            DataError::InvalidValue => write!(f, "Value must be finite"),
            DataError::EmptyCategory => write!(f, "Category cannot be empty"),
            DataError::TransformationError(msg) => write!(f, "Transformation failed: {}", msg),
        }
    }
}

impl Error for DataError {}

impl DataPoint {
    pub fn new(timestamp: i64, value: f64, category: String) -> Result<Self, DataError> {
        if timestamp <= 0 {
            return Err(DataError::InvalidTimestamp);
        }
        
        if !value.is_finite() {
            return Err(DataError::InvalidValue);
        }
        
        if category.trim().is_empty() {
            return Err(DataError::EmptyCategory);
        }
        
        Ok(Self {
            timestamp,
            value,
            category,
        })
    }
    
    pub fn transform(&self, multiplier: f64) -> Result<Self, DataError> {
        if !multiplier.is_finite() || multiplier == 0.0 {
            return Err(DataError::TransformationError(
                "Multiplier must be finite and non-zero".to_string()
            ));
        }
        
        let new_value = self.value * multiplier;
        
        Self::new(self.timestamp, new_value, self.category.clone())
    }
    
    pub fn normalize(&self, min: f64, max: f64) -> Result<f64, DataError> {
        if min >= max {
            return Err(DataError::TransformationError(
                "Min must be less than max".to_string()
            ));
        }
        
        if !min.is_finite() || !max.is_finite() {
            return Err(DataError::TransformationError(
                "Bounds must be finite".to_string()
            ));
        }
        
        let normalized = (self.value - min) / (max - min);
        
        if normalized.is_finite() {
            Ok(normalized)
        } else {
            Err(DataError::TransformationError(
                "Normalization produced non-finite result".to_string()
            ))
        }
    }
}

pub struct DataProcessor {
    points: Vec<DataPoint>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self { points: Vec::new() }
    }
    
    pub fn add_point(&mut self, point: DataPoint) {
        self.points.push(point);
    }
    
    pub fn process_all(&self, operation: fn(&DataPoint) -> Result<f64, DataError>) -> Vec<Result<f64, DataError>> {
        self.points.iter().map(operation).collect()
    }
    
    pub fn filter_by_category(&self, category: &str) -> Vec<&DataPoint> {
        self.points
            .iter()
            .filter(|p| p.category == category)
            .collect()
    }
    
    pub fn calculate_statistics(&self) -> Option<(f64, f64, f64)> {
        if self.points.is_empty() {
            return None;
        }
        
        let sum: f64 = self.points.iter().map(|p| p.value).sum();
        let count = self.points.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.points
            .iter()
            .map(|p| (p.value - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        Some((mean, variance, std_dev))
    }
}