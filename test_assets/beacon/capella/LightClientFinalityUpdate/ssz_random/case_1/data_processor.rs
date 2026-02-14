
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
    InvalidValue(f64),
    MissingMetadata(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::EmptyValues => write!(f, "Record contains no values"),
            DataError::InvalidValue(val) => write!(f, "Invalid value detected: {}", val),
            DataError::MissingMetadata(key) => write!(f, "Missing metadata key: {}", key),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    validation_threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Self {
        DataProcessor {
            records: Vec::new(),
            validation_threshold: threshold,
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        self.validate_record(&record)?;
        self.records.push(record);
        Ok(())
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }

        if record.values.is_empty() {
            return Err(DataError::EmptyValues);
        }

        for &value in &record.values {
            if value.is_nan() || value.is_infinite() {
                return Err(DataError::InvalidValue(value));
            }
        }

        Ok(())
    }

    pub fn process_records(&mut self) -> HashMap<u32, f64> {
        let mut results = HashMap::new();

        for record in &self.records {
            if let Some(avg) = self.calculate_average(&record.values) {
                if avg > self.validation_threshold {
                    results.insert(record.id, avg);
                }
            }
        }

        results
    }

    fn calculate_average(&self, values: &[f64]) -> Option<f64> {
        if values.is_empty() {
            return None;
        }

        let sum: f64 = values.iter().sum();
        Some(sum / values.len() as f64)
    }

    pub fn filter_by_metadata(&self, key: &str, value: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| {
                record
                    .metadata
                    .get(key)
                    .map_or(false, |v| v == value)
            })
            .collect()
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_processing() {
        let mut processor = DataProcessor::new(10.0);
        let mut metadata = HashMap::new();
        metadata.insert("category".to_string(), "test".to_string());

        let record = DataRecord {
            id: 1,
            values: vec![5.0, 10.0, 15.0],
            metadata,
        };

        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.get_record_count(), 1);
    }

    #[test]
    fn test_invalid_record_rejection() {
        let mut processor = DataProcessor::new(10.0);
        let record = DataRecord {
            id: 0,
            values: vec![5.0, 10.0],
            metadata: HashMap::new(),
        };

        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_average_calculation() {
        let processor = DataProcessor::new(10.0);
        let values = vec![2.0, 4.0, 6.0];
        let avg = processor.calculate_average(&values).unwrap();
        assert_eq!(avg, 4.0);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    filters: Vec<Box<dyn Fn(&HashMap<String, String>) -> bool>>,
    transformers: Vec<Box<dyn Fn(HashMap<String, String>) -> HashMap<String, String>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            filters: Vec::new(),
            transformers: Vec::new(),
        }
    }

    pub fn add_filter<F>(&mut self, filter: F)
    where
        F: Fn(&HashMap<String, String>) -> bool + 'static,
    {
        self.filters.push(Box::new(filter));
    }

    pub fn add_transformer<F>(&mut self, transformer: F)
    where
        F: Fn(HashMap<String, String>) -> HashMap<String, String> + 'static,
    {
        self.transformers.push(Box::new(transformer));
    }

    pub fn process(&self, mut data: HashMap<String, String>) -> Option<HashMap<String, String>> {
        for filter in &self.filters {
            if !filter(&data) {
                return None;
            }
        }

        for transformer in &self.transformers {
            data = transformer(data);
        }

        Some(data)
    }

    pub fn process_batch(&self, batch: Vec<HashMap<String, String>>) -> Vec<HashMap<String, String>> {
        batch
            .into_iter()
            .filter_map(|item| self.process(item))
            .collect()
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();
    
    processor.add_filter(|data| {
        data.contains_key("id") && !data.get("id").unwrap().is_empty()
    });

    processor.add_transformer(|mut data| {
        data.entry("processed".to_string())
            .or_insert("true".to_string());
        data
    });

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor() {
        let processor = create_default_processor();
        
        let mut valid_data = HashMap::new();
        valid_data.insert("id".to_string(), "123".to_string());
        valid_data.insert("name".to_string(), "test".to_string());

        let result = processor.process(valid_data);
        assert!(result.is_some());
        
        let processed_data = result.unwrap();
        assert_eq!(processed_data.get("processed"), Some(&"true".to_string()));
        assert_eq!(processed_data.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_invalid_data() {
        let processor = create_default_processor();
        
        let mut invalid_data = HashMap::new();
        invalid_data.insert("name".to_string(), "test".to_string());

        let result = processor.process(invalid_data);
        assert!(result.is_none());
    }

    #[test]
    fn test_batch_processing() {
        let processor = create_default_processor();
        
        let batch = vec![
            {
                let mut map = HashMap::new();
                map.insert("id".to_string(), "1".to_string());
                map
            },
            {
                let mut map = HashMap::new();
                map.insert("name".to_string(), "no_id".to_string());
                map
            },
            {
                let mut map = HashMap::new();
                map.insert("id".to_string(), "2".to_string());
                map
            },
        ];

        let results = processor.process_batch(batch);
        assert_eq!(results.len(), 2);
        
        for result in results {
            assert!(result.contains_key("id"));
            assert_eq!(result.get("processed"), Some(&"true".to_string()));
        }
    }
}
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
        
        let mut loaded_count = 0;
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                return Err(format!("Invalid format at line {}", line_num + 1).into());
            }
            
            let id = parts[0].parse::<u32>()?;
            let value = parts[1].parse::<f64>()?;
            let category = parts[2].trim();
            
            match DataRecord::new(id, value, category) {
                Ok(record) => {
                    self.records.push(record);
                    loaded_count += 1;
                }
                Err(e) => {
                    eprintln!("Warning: Skipping line {}: {}", line_num + 1, e);
                }
            }
        }
        
        Ok(loaded_count)
    }
    
    pub fn total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }
    
    pub fn average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            None
        } else {
            Some(self.total_value() / self.records.len() as f64)
        }
    }
    
    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
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
    fn test_record_creation() {
        let record = DataRecord::new(1, 42.5, "test").unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
    }
    
    #[test]
    fn test_invalid_record() {
        assert!(DataRecord::new(1, -5.0, "test").is_err());
        assert!(DataRecord::new(1, 5.0, "").is_err());
    }
    
    #[test]
    fn test_data_processor() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "1,10.5,category_a")?;
        writeln!(temp_file, "2,20.0,category_b")?;
        writeln!(temp_file, "3,15.75,category_a")?;
        
        let mut processor = DataProcessor::new();
        let count = processor.load_from_csv(temp_file.path())?;
        
        assert_eq!(count, 3);
        assert_eq!(processor.total_value(), 46.25);
        assert_eq!(processor.average_value(), Some(15.416666666666666));
        
        let category_a = processor.filter_by_category("category_a");
        assert_eq!(category_a.len(), 2);
        
        Ok(())
    }
}