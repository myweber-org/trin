
use std::collections::HashMap;

pub struct DataProcessor {
    validators: HashMap<String, Box<dyn Fn(&str) -> bool>>,
    transformers: HashMap<String, Box<dyn Fn(String) -> String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validators: HashMap::new(),
            transformers: HashMap::new(),
        }
    }

    pub fn register_validator(&mut self, name: &str, validator: Box<dyn Fn(&str) -> bool>) {
        self.validators.insert(name.to_string(), validator);
    }

    pub fn register_transformer(&mut self, name: &str, transformer: Box<dyn Fn(String) -> String>) {
        self.transformers.insert(name.to_string(), transformer);
    }

    pub fn validate(&self, name: &str, data: &str) -> bool {
        self.validators
            .get(name)
            .map_or(false, |validator| validator(data))
    }

    pub fn transform(&self, name: &str, data: String) -> Option<String> {
        self.transformers
            .get(name)
            .map(|transformer| transformer(data))
    }

    pub fn process_pipeline(&self, data: String, steps: &[(&str, &str)]) -> Result<String, String> {
        let mut current_data = data;

        for (step_type, step_name) in steps {
            match *step_type {
                "validate" => {
                    if !self.validate(step_name, &current_data) {
                        return Err(format!("Validation failed at step: {}", step_name));
                    }
                }
                "transform" => {
                    current_data = self.transform(step_name, current_data)
                        .ok_or_else(|| format!("Transformer not found: {}", step_name))?;
                }
                _ => return Err(format!("Unknown step type: {}", step_type)),
            }
        }

        Ok(current_data)
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.register_validator("non_empty", Box::new(|s| !s.trim().is_empty()));
    processor.register_validator("is_numeric", Box::new(|s| s.parse::<f64>().is_ok()));
    
    processor.register_transformer("to_uppercase", Box::new(|s| s.to_uppercase()));
    processor.register_transformer("trim_spaces", Box::new(|s| s.trim().to_string()));
    processor.register_transformer("reverse_string", Box::new(|s| s.chars().rev().collect()));

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let processor = create_default_processor();
        assert!(processor.validate("non_empty", "test"));
        assert!(!processor.validate("non_empty", "   "));
        assert!(processor.validate("is_numeric", "123.45"));
        assert!(!processor.validate("is_numeric", "abc"));
    }

    #[test]
    fn test_transformation() {
        let processor = create_default_processor();
        assert_eq!(processor.transform("to_uppercase", "hello".to_string()), Some("HELLO".to_string()));
        assert_eq!(processor.transform("trim_spaces", "  test  ".to_string()), Some("test".to_string()));
        assert_eq!(processor.transform("reverse_string", "abc".to_string()), Some("cba".to_string()));
    }

    #[test]
    fn test_pipeline() {
        let processor = create_default_processor();
        let steps = vec![
            ("validate", "non_empty"),
            ("transform", "to_uppercase"),
            ("transform", "reverse_string"),
        ];
        
        let result = processor.process_pipeline("hello".to_string(), &steps);
        assert_eq!(result, Ok("OLLEH".to_string()));
        
        let invalid_result = processor.process_pipeline("   ".to_string(), &steps);
        assert!(invalid_result.is_err());
    }
}use csv::Reader;
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
        let mut reader = Reader::from_path(path)?;
        for result in reader.deserialize() {
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

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&Record>> {
        let mut groups = std::collections::HashMap::new();
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }
        groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,20.0,Category2").unwrap();
        
        processor.load_from_csv(temp_file.path()).unwrap();
        
        assert_eq!(processor.records.len(), 2);
        assert_eq!(processor.calculate_average(), Some(15.25));
        
        let valid = processor.validate_records();
        assert_eq!(valid.len(), 2);
        
        let groups = processor.group_by_category();
        assert_eq!(groups.len(), 2);
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

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);

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
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }
        
        groups
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        
        if values.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        (mean, min, max)
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
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,20.3,Category2").unwrap();
        writeln!(temp_file, "3,ItemC,15.7,Category1").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let valid_records = processor.validate_records();
        assert_eq!(valid_records.len(), 3);
        
        let total = processor.calculate_total();
        assert!((total - 46.5).abs() < 0.001);
        
        let groups = processor.group_by_category();
        assert_eq!(groups.get("Category1").unwrap().len(), 2);
        assert_eq!(groups.get("Category2").unwrap().len(), 1);
        
        let (mean, min, max) = processor.get_statistics();
        assert!((mean - 15.5).abs() < 0.001);
        assert!((min - 10.5).abs() < 0.001);
        assert!((max - 20.3).abs() < 0.001);
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
    InvalidValue,
    MissingField,
    CategoryNotFound,
    TransformationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue => write!(f, "Invalid numeric value"),
            ProcessingError::MissingField => write!(f, "Required field is missing"),
            ProcessingError::CategoryNotFound => write!(f, "Category does not exist"),
            ProcessingError::TransformationError(msg) => write!(f, "Transformation failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    categories: HashMap<String, Vec<String>>,
    validation_rules: Vec<Box<dyn Fn(&DataRecord) -> Result<(), ProcessingError>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            categories: HashMap::new(),
            validation_rules: Vec::new(),
        }
    }

    pub fn add_category(&mut self, category: String, allowed_values: Vec<String>) {
        self.categories.insert(category, allowed_values);
    }

    pub fn add_validation_rule<F>(&mut self, rule: F)
    where
        F: Fn(&DataRecord) -> Result<(), ProcessingError> + 'static,
    {
        self.validation_rules.push(Box::new(rule));
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.id == 0 {
            return Err(ProcessingError::InvalidValue);
        }

        if record.name.is_empty() {
            return Err(ProcessingError::MissingField);
        }

        if let Some(allowed_values) = self.categories.get(&record.category) {
            if !allowed_values.contains(&record.category) {
                return Err(ProcessingError::CategoryNotFound);
            }
        }

        for rule in &self.validation_rules {
            rule(record)?;
        }

        Ok(())
    }

    pub fn transform_record(
        &self,
        record: &DataRecord,
        transformer: impl Fn(&DataRecord) -> Result<DataRecord, ProcessingError>,
    ) -> Result<DataRecord, ProcessingError> {
        self.validate_record(record)?;
        transformer(record)
    }

    pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if records.is_empty() {
            return stats;
        }

        let sum: f64 = records.iter().map(|r| r.value).sum();
        let count = records.len() as f64;
        let avg = sum / count;
        
        let min = records.iter().map(|r| r.value).fold(f64::INFINITY, f64::min);
        let max = records.iter().map(|r| r.value).fold(f64::NEG_INFINITY, f64::max);

        stats.insert("total_records".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("average".to_string(), avg);
        stats.insert("minimum".to_string(), min);
        stats.insert("maximum".to_string(), max);

        stats
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
    fn test_record_validation() {
        let mut processor = DataProcessor::new();
        processor.add_category("type_a".to_string(), vec!["type_a".to_string()]);

        let valid_record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 42.0,
            category: "type_a".to_string(),
        };

        assert!(processor.validate_record(&valid_record).is_ok());

        let invalid_record = DataRecord {
            id: 0,
            name: "".to_string(),
            value: -1.0,
            category: "invalid".to_string(),
        };

        assert!(processor.validate_record(&invalid_record).is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            DataRecord {
                id: 1,
                name: "A".to_string(),
                value: 10.0,
                category: "test".to_string(),
            },
            DataRecord {
                id: 2,
                name: "B".to_string(),
                value: 20.0,
                category: "test".to_string(),
            },
            DataRecord {
                id: 3,
                name: "C".to_string(),
                value: 30.0,
                category: "test".to_string(),
            },
        ];

        let stats = DataProcessor::calculate_statistics(&records);
        
        assert_eq!(stats.get("total_records"), Some(&3.0));
        assert_eq!(stats.get("sum"), Some(&60.0));
        assert_eq!(stats.get("average"), Some(&20.0));
        assert_eq!(stats.get("minimum"), Some(&10.0));
        assert_eq!(stats.get("maximum"), Some(&30.0));
    }
}