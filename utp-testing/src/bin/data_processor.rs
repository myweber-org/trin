use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
struct DataRecord {
    id: u32,
    value: f64,
    timestamp: u64,
}

#[derive(Debug)]
enum ValidationError {
    InvalidId,
    InvalidValue,
    InvalidTimestamp,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            ValidationError::InvalidTimestamp => write!(f, "Timestamp must be in the past"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    fn new(id: u32, value: f64, timestamp: u64) -> Result<Self, ValidationError> {
        if id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if value < 0.0 || value > 1000.0 {
            return Err(ValidationError::InvalidValue);
        }
        
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if timestamp > current_time {
            return Err(ValidationError::InvalidTimestamp);
        }
        
        Ok(Self { id, value, timestamp })
    }
    
    fn normalize_value(&self) -> f64 {
        self.value / 1000.0
    }
    
    fn is_anomaly(&self, threshold: f64) -> bool {
        self.normalize_value() > threshold
    }
}

struct DataProcessor {
    records: Vec<DataRecord>,
    anomaly_threshold: f64,
}

impl DataProcessor {
    fn new(threshold: f64) -> Self {
        Self {
            records: Vec::new(),
            anomaly_threshold: threshold,
        }
    }
    
    fn add_record(&mut self, id: u32, value: f64, timestamp: u64) -> Result<(), ValidationError> {
        let record = DataRecord::new(id, value, timestamp)?;
        self.records.push(record);
        Ok(())
    }
    
    fn process_records(&self) -> (Vec<f64>, Vec<DataRecord>) {
        let mut normalized_values = Vec::new();
        let mut anomalies = Vec::new();
        
        for record in &self.records {
            normalized_values.push(record.normalize_value());
            
            if record.is_anomaly(self.anomaly_threshold) {
                anomalies.push(record.clone());
            }
        }
        
        (normalized_values, anomalies)
    }
    
    fn calculate_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let count = values.len() as f64;
        let sum: f64 = values.iter().sum();
        let mean = sum / count;
        
        let variance: f64 = values.iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
}

fn transform_data(records: &[DataRecord]) -> Vec<(u32, f64)> {
    records.iter()
        .map(|r| (r.id, r.normalize_value()))
        .collect()
}

fn main() {
    let mut processor = DataProcessor::new(0.8);
    
    let sample_data = vec![
        (1, 450.5, 1672531200),
        (2, 850.2, 1672534800),
        (3, 920.7, 1672538400),
        (4, 150.3, 1672542000),
    ];
    
    for (id, value, timestamp) in sample_data {
        match processor.add_record(id, value, timestamp) {
            Ok(_) => println!("Record {} added successfully", id),
            Err(e) => println!("Failed to add record {}: {}", id, e),
        }
    }
    
    let (normalized, anomalies) = processor.process_records();
    println!("Normalized values: {:?}", normalized);
    println!("Anomalies detected: {}", anomalies.len());
    
    let (mean, variance, std_dev) = processor.calculate_statistics();
    println!("Statistics - Mean: {:.2}, Variance: {:.2}, Std Dev: {:.2}", 
             mean, variance, std_dev);
    
    let transformed = transform_data(&processor.records);
    println!("Transformed data: {:?}", transformed);
}