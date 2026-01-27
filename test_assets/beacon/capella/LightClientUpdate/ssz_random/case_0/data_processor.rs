
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
        if let Some(value) = data.get("timestamp") {
            if let Ok(parsed) = value.parse::<i64>() {
                let formatted = format!("{}", parsed);
                data.insert("formatted_timestamp".to_string(), formatted);
            }
        }
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
        valid_data.insert("timestamp".to_string(), "1625097600".to_string());

        let result = processor.process(valid_data);
        assert!(result.is_some());
        let processed = result.unwrap();
        assert_eq!(processed.get("formatted_timestamp"), Some(&"1625097600".to_string()));

        let mut invalid_data = HashMap::new();
        invalid_data.insert("timestamp".to_string(), "1625097600".to_string());

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
                map.insert("timestamp".to_string(), "100".to_string());
                map
            },
            {
                let mut map = HashMap::new();
                map.insert("timestamp".to_string(), "200".to_string());
                map
            },
            {
                let mut map = HashMap::new();
                map.insert("id".to_string(), "3".to_string());
                map.insert("timestamp".to_string(), "300".to_string());
                map
            },
        ];

        let results = processor.process_batch(batch);
        assert_eq!(results.len(), 2);
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
pub enum ProcessingError {
    InvalidData(String),
    TransformationError(String),
    ValidationFailed(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
            ProcessingError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    normalization_factor: f64,
    validation_threshold: f64,
}

impl DataProcessor {
    pub fn new(normalization_factor: f64, validation_threshold: f64) -> Self {
        DataProcessor {
            normalization_factor,
            validation_threshold,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.is_empty() {
            return Err(ProcessingError::InvalidData("Empty values vector".to_string()));
        }

        for (i, &value) in record.values.iter().enumerate() {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::InvalidData(
                    format!("Invalid value at index {}: {}", i, value)
                ));
            }
        }

        if !record.metadata.contains_key("source") {
            return Err(ProcessingError::ValidationFailed(
                "Missing source metadata".to_string()
            ));
        }

        Ok(())
    }

    pub fn normalize_values(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        self.validate_record(record)?;

        for value in &mut record.values {
            *value = *value / self.normalization_factor;
            
            if value.abs() > self.validation_threshold {
                return Err(ProcessingError::TransformationError(
                    format!("Normalized value {} exceeds threshold", value)
                ));
            }
        }

        record.metadata.insert(
            "normalized".to_string(),
            "true".to_string()
        );

        Ok(())
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> Result<HashMap<String, f64>, ProcessingError> {
        if records.is_empty() {
            return Err(ProcessingError::InvalidData("No records provided".to_string()));
        }

        let mut stats = HashMap::new();
        let mut sum = 0.0;
        let mut count = 0;

        for record in records {
            self.validate_record(record)?;
            for &value in &record.values {
                sum += value;
                count += 1;
            }
        }

        if count > 0 {
            let mean = sum / count as f64;
            stats.insert("mean".to_string(), mean);
            stats.insert("total_records".to_string(), records.len() as f64);
            stats.insert("total_values".to_string(), count as f64);
        }

        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(1.0, 100.0);
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 1,
            values: vec![1.0, 2.0, 3.0],
            metadata,
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_normalization() {
        let processor = DataProcessor::new(2.0, 10.0);
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let mut record = DataRecord {
            id: 1,
            values: vec![4.0, 6.0, 8.0],
            metadata,
        };

        assert!(processor.normalize_values(&mut record).is_ok());
        assert_eq!(record.values, vec![2.0, 3.0, 4.0]);
        assert_eq!(record.metadata.get("normalized"), Some(&"true".to_string()));
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(1.0, 100.0);
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let records = vec![
            DataRecord {
                id: 1,
                values: vec![1.0, 2.0],
                metadata: metadata.clone(),
            },
            DataRecord {
                id: 2,
                values: vec![3.0, 4.0],
                metadata,
            },
        ];

        let stats = processor.calculate_statistics(&records).unwrap();
        assert_eq!(stats.get("mean"), Some(&2.5));
        assert_eq!(stats.get("total_records"), Some(&2.0));
        assert_eq!(stats.get("total_values"), Some(&4.0));
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

    pub fn process_dataset(&mut self, dataset_name: &str, data: Vec<f64>) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        for rule in &self.validation_rules {
            if rule.required && data.iter().any(|&x| x.is_nan()) {
                return Err(format!("Field {} contains invalid values", rule.field_name));
            }

            if let Some(&value) = data.iter().find(|&&x| x < rule.min_value || x > rule.max_value) {
                return Err(format!("Value {} out of range for field {}", value, rule.field_name));
            }
        }

        let processed_data: Vec<f64> = data
            .iter()
            .map(|&x| x * 2.0)
            .filter(|&x| x > 0.0)
            .collect();

        self.cache.insert(dataset_name.to_string(), processed_data.clone());

        Ok(processed_data)
    }

    pub fn get_cached_data(&self, dataset_name: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_name)
    }

    pub fn calculate_statistics(&self, dataset_name: &str) -> Option<Statistics> {
        self.cache.get(dataset_name).map(|data| {
            let sum: f64 = data.iter().sum();
            let count = data.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = data.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;
            
            Statistics {
                mean,
                variance,
                count: data.len(),
                min: *data.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0),
                max: *data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0),
            }
        })
    }
}

pub struct Statistics {
    pub mean: f64,
    pub variance: f64,
    pub count: usize,
    pub min: f64,
    pub max: f64,
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
        processor.add_validation_rule(ValidationRule::new("temperature", -50.0, 100.0, true));
        
        let data = vec![20.5, 25.0, 30.2, 15.8];
        let result = processor.process_dataset("weather", data);
        
        assert!(result.is_ok());
        assert_eq!(processor.get_cached_data("weather").unwrap().len(), 4);
    }

    #[test]
    fn test_invalid_data() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(ValidationRule::new("pressure", 0.0, 10.0, true));
        
        let data = vec![5.0, 15.0, 3.0];
        let result = processor.process_dataset("pressure_data", data);
        
        assert!(result.is_err());
    }
}use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug)]
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.records() {
            let record = result?;
            if record.len() != 3 {
                continue;
            }

            let id: u32 = record[0].parse()?;
            let value: f64 = record[1].parse()?;
            let category = &record[2];

            match DataRecord::new(id, value, category) {
                Ok(data_record) => self.records.push(data_record),
                Err(e) => eprintln!("Skipping invalid record: {}", e),
            }
        }

        Ok(())
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average().unwrap_or(0.0);

        (min, max, avg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert!(processor.calculate_average().is_none());
        assert_eq!(processor.get_statistics(), (0.0, 0.0, 0.0));
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

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
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
            
            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Vec<usize> {
        let mut invalid_indices = Vec::new();
        
        for (index, record) in records.iter().enumerate() {
            if record.is_empty() || record.iter().any(|field| field.is_empty()) {
                invalid_indices.push(index);
            }
        }
        
        invalid_indices
    }

    pub fn extract_column(&self, records: &[Vec<String>], column_index: usize) -> Vec<String> {
        records
            .iter()
            .filter_map(|record| record.get(column_index).cloned())
            .collect()
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
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Jane,25,London").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_validate_records() {
        let records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["".to_string(), "b".to_string()],
            vec!["a".to_string()],
            vec![],
        ];
        
        let processor = DataProcessor::new(',', false);
        let invalid = processor.validate_records(&records);
        
        assert_eq!(invalid, vec![1, 3]);
    }

    #[test]
    fn test_extract_column() {
        let records = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["d".to_string(), "e".to_string(), "f".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let column = processor.extract_column(&records, 1);
        
        assert_eq!(column, vec!["b".to_string(), "e".to_string()]);
    }
}