use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        
        Ok(Self {
            id,
            value,
            category: category.to_string(),
        })
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
        Self {
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
                Ok(id) => id,
                Err(_) => continue,
            };
            
            let value = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => continue,
            };
            
            let category = parts[2].trim();
            
            match DataRecord::new(id, value, category) {
                Ok(record) => {
                    self.records.push(record);
                    count += 1;
                }
                Err(_) => continue,
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
    
    pub fn get_average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let total: f64 = self.records.iter().map(|record| record.value).sum();
        Some(total / self.records.len() as f64)
    }
    
    pub fn sort_by_value(&mut self) {
        self.records.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap());
    }
    
    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test").unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
    }
    
    #[test]
    fn test_invalid_data_record() {
        assert!(DataRecord::new(1, -5.0, "test").is_err());
        assert!(DataRecord::new(1, 5.0, "").is_err());
    }
    
    #[test]
    fn test_calculate_adjusted_value() {
        let record = DataRecord::new(1, 10.0, "test").unwrap();
        assert_eq!(record.calculate_adjusted_value(2.0), 20.0);
    }
    
    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,category_a").unwrap();
        writeln!(temp_file, "2,20.3,category_b").unwrap();
        writeln!(temp_file, "3,15.7,category_a").unwrap();
        
        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        
        let filtered = processor.filter_by_category("category_a");
        assert_eq!(filtered.len(), 2);
        
        let total = processor.calculate_total_value();
        assert_eq!(total, 46.5);
        
        let average = processor.get_average_value().unwrap();
        assert!((average - 15.5).abs() < 0.01);
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
            if let Some(first_field) = record.get(0) {
                if let Ok(value) = first_field.parse::<f64>() {
                    values.push(value);
                }
            }
        }

        Ok(DataSet { values })
    }

    pub fn add_value(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.values.is_empty() {
            return None;
        }
        let sum: f64 = self.values.iter().sum();
        Some(sum / self.values.len() as f64)
    }

    pub fn calculate_std_dev(&self) -> Option<f64> {
        if self.values.len() < 2 {
            return None;
        }
        
        let mean = self.calculate_mean()?;
        let variance: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (self.values.len() - 1) as f64;
        
        Some(variance.sqrt())
    }

    pub fn get_summary(&self) -> DataSummary {
        DataSummary {
            count: self.values.len(),
            mean: self.calculate_mean(),
            std_dev: self.calculate_std_dev(),
            min: self.values.iter().copied().reduce(f64::min),
            max: self.values.iter().copied().reduce(f64::max),
        }
    }
}

pub struct DataSummary {
    pub count: usize,
    pub mean: Option<f64>,
    pub std_dev: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl std::fmt::Display for DataSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Data Summary:")?;
        writeln!(f, "  Count: {}", self.count)?;
        
        if let Some(mean) = self.mean {
            writeln!(f, "  Mean: {:.4}", mean)?;
        } else {
            writeln!(f, "  Mean: N/A")?;
        }
        
        if let Some(std_dev) = self.std_dev {
            writeln!(f, "  Std Dev: {:.4}", std_dev)?;
        } else {
            writeln!(f, "  Std Dev: N/A")?;
        }
        
        if let Some(min) = self.min {
            writeln!(f, "  Min: {:.4}", min)?;
        } else {
            writeln!(f, "  Min: N/A")?;
        }
        
        if let Some(max) = self.max {
            writeln!(f, "  Max: {:.4}", max)?;
        } else {
            writeln!(f, "  Max: N/A")?;
        }
        
        Ok(())
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

    pub fn process_csv<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
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

    pub fn validate_numeric_fields(&self, data: &[Vec<String>], column_index: usize) -> Result<Vec<f64>, Box<dyn Error>> {
        let mut numeric_values = Vec::new();
        
        for (row_index, row) in data.iter().enumerate() {
            if column_index < row.len() {
                let value = &row[column_index];
                match value.parse::<f64>() {
                    Ok(num) => numeric_values.push(num),
                    Err(_) => return Err(format!("Invalid numeric value at row {}: {}", row_index + 1, value).into()),
                }
            } else {
                return Err(format!("Column index {} out of bounds at row {}", column_index, row_index + 1).into());
            }
        }
        
        Ok(numeric_values)
    }

    pub fn calculate_statistics(&self, numbers: &[f64]) -> (f64, f64, f64) {
        if numbers.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = numbers.iter().sum();
        let mean = sum / numbers.len() as f64;
        
        let variance: f64 = numbers.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / numbers.len() as f64;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.5").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_csv(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "50000.5"]);
    }

    #[test]
    fn test_numeric_validation() {
        let data = vec![
            vec!["10.5".to_string(), "text".to_string()],
            vec!["20.0".to_string(), "more".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let numbers = processor.validate_numeric_fields(&data, 0).unwrap();
        
        assert_eq!(numbers, vec![10.5, 20.0]);
    }

    #[test]
    fn test_statistics_calculation() {
        let numbers = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let processor = DataProcessor::new(',', false);
        let (mean, variance, std_dev) = processor.calculate_statistics(&numbers);
        
        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert_eq!(std_dev, 2.0_f64.sqrt());
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum DataError {
    InvalidFormat,
    OutOfRange,
    MissingField,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidFormat => write!(f, "Data format is invalid"),
            DataError::OutOfRange => write!(f, "Value is out of acceptable range"),
            DataError::MissingField => write!(f, "Required field is missing"),
        }
    }
}

impl Error for DataError {}

pub struct DataPoint {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

impl DataPoint {
    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::MissingField);
        }
        
        if !self.value.is_finite() || self.value < 0.0 || self.value > 1000.0 {
            return Err(DataError::OutOfRange);
        }
        
        if self.timestamp <= 0 {
            return Err(DataError::InvalidFormat);
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) -> Result<(), DataError> {
        self.validate()?;
        
        if !multiplier.is_finite() || multiplier <= 0.0 {
            return Err(DataError::InvalidFormat);
        }
        
        self.value *= multiplier;
        
        if self.value > 1000.0 {
            return Err(DataError::OutOfRange);
        }
        
        Ok(())
    }
}

pub fn process_data_points(points: &mut [DataPoint], multiplier: f64) -> Result<Vec<DataPoint>, DataError> {
    let mut results = Vec::with_capacity(points.len());
    
    for point in points {
        let mut processed_point = DataPoint {
            id: point.id,
            value: point.value,
            timestamp: point.timestamp,
        };
        
        processed_point.transform(multiplier)?;
        results.push(processed_point);
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_data_point() {
        let point = DataPoint {
            id: 1,
            value: 50.0,
            timestamp: 1625097600,
        };
        
        assert!(point.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_id() {
        let point = DataPoint {
            id: 0,
            value: 50.0,
            timestamp: 1625097600,
        };
        
        assert!(matches!(point.validate(), Err(DataError::MissingField)));
    }
    
    #[test]
    fn test_transform_valid() {
        let mut point = DataPoint {
            id: 1,
            value: 50.0,
            timestamp: 1625097600,
        };
        
        assert!(point.transform(2.0).is_ok());
        assert_eq!(point.value, 100.0);
    }
}