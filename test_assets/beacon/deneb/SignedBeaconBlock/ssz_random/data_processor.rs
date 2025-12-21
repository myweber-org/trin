
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>) -> Result<Self, Box<dyn Error>> {
        if values.is_empty() {
            return Err("Values cannot be empty".into());
        }
        if values.iter().any(|&v| v.is_nan() || v.is_infinite()) {
            return Err("Values contain invalid numbers".into());
        }
        
        Ok(Self {
            id,
            values,
            metadata: HashMap::new(),
        })
    }
    
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    pub fn calculate_statistics(&self) -> Statistics {
        let sum: f64 = self.values.iter().sum();
        let count = self.values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.values
            .iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        let min = self.values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        Statistics {
            mean,
            std_dev,
            min,
            max,
            count: self.values.len(),
        }
    }
    
    pub fn normalize(&mut self) {
        let stats = self.calculate_statistics();
        if stats.std_dev > 0.0 {
            for value in &mut self.values {
                *value = (*value - stats.mean) / stats.std_dev;
            }
        }
    }
}

#[derive(Debug)]
pub struct Statistics {
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
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
    
    pub fn add_record(&mut self, record: DataRecord) -> Result<(), Box<dyn Error>> {
        if self.records.iter().any(|r| r.id == record.id) {
            return Err(format!("Record with id {} already exists", record.id).into());
        }
        self.records.push(record);
        Ok(())
    }
    
    pub fn process_all(&mut self) {
        for record in &mut self.records {
            record.normalize();
        }
    }
    
    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.iter().find(|r| r.id == id)
    }
    
    pub fn aggregate_statistics(&self) -> Option<Statistics> {
        if self.records.is_empty() {
            return None;
        }
        
        let all_values: Vec<f64> = self.records
            .iter()
            .flat_map(|r| r.values.clone())
            .collect();
        
        let sum: f64 = all_values.iter().sum();
        let count = all_values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = all_values
            .iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        let min = all_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = all_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        Some(Statistics {
            mean,
            std_dev,
            min,
            max,
            count: all_values.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0]).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.values, vec![1.0, 2.0, 3.0]);
    }
    
    #[test]
    fn test_invalid_record() {
        let result = DataRecord::new(1, vec![]);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        let stats = record.calculate_statistics();
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.count, 5);
    }
    
    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        let record1 = DataRecord::new(1, vec![1.0, 2.0, 3.0]).unwrap();
        let record2 = DataRecord::new(2, vec![4.0, 5.0, 6.0]).unwrap();
        
        processor.add_record(record1).unwrap();
        processor.add_record(record2).unwrap();
        
        assert_eq!(processor.get_record(1).unwrap().id, 1);
        assert_eq!(processor.get_record(2).unwrap().id, 2);
    }
}