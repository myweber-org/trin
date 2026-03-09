use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    frequency_map: HashMap<String, u32>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            frequency_map: HashMap::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines().skip(1) {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if parts.len() >= 2 {
                if let Ok(value) = parts[1].parse::<f64>() {
                    self.data.push(value);
                }
                
                let category = parts[0].to_string();
                *self.frequency_map.entry(category).or_insert(0) += 1;
            }
        }
        
        Ok(())
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        
        let sum: f64 = self.data.iter().sum();
        Some(sum / self.data.len() as f64)
    }

    pub fn calculate_median(&mut self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        
        self.data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = self.data.len() / 2;
        
        if self.data.len() % 2 == 0 {
            Some((self.data[mid - 1] + self.data[mid]) / 2.0)
        } else {
            Some(self.data[mid])
        }
    }

    pub fn get_frequency_distribution(&self) -> &HashMap<String, u32> {
        &self.frequency_map
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&x| x > threshold)
            .cloned()
            .collect()
    }

    pub fn data_summary(&self) -> String {
        let mean = self.calculate_mean().unwrap_or(0.0);
        let count = self.data.len();
        let unique_categories = self.frequency_map.len();
        
        format!(
            "Data Summary:\nTotal entries: {}\nUnique categories: {}\nMean value: {:.2}",
            count, unique_categories, mean
        )
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
        writeln!(temp_file, "category,value").unwrap();
        writeln!(temp_file, "A,10.5").unwrap();
        writeln!(temp_file, "B,20.3").unwrap();
        writeln!(temp_file, "A,15.7").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        assert_eq!(processor.calculate_mean(), Some(15.5));
        assert_eq!(processor.filter_by_threshold(15.0).len(), 2);
        
        let freq = processor.get_frequency_distribution();
        assert_eq!(freq.get("A"), Some(&2));
        assert_eq!(freq.get("B"), Some(&1));
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
    pub fn new(id: u32, timestamp: i64, values: Vec<f64>) -> Self {
        DataRecord {
            id,
            timestamp,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.id == 0 {
            return Err("Invalid record ID".into());
        }
        
        if self.timestamp < 0 {
            return Err("Timestamp cannot be negative".into());
        }
        
        if self.values.is_empty() {
            return Err("Values array cannot be empty".into());
        }
        
        for value in &self.values {
            if !value.is_finite() {
                return Err("Values must be finite numbers".into());
            }
        }
        
        Ok(())
    }
}

pub fn transform_values(record: &mut DataRecord, multiplier: f64) {
    for value in &mut record.values {
        *value *= multiplier;
    }
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if records.is_empty() {
        return stats;
    }
    
    let total_records = records.len() as f64;
    let mut value_sum = 0.0;
    let mut value_count = 0;
    
    for record in records {
        for value in &record.values {
            value_sum += value;
            value_count += 1;
        }
    }
    
    stats.insert("total_records".to_string(), total_records);
    stats.insert("average_value".to_string(), value_sum / value_count as f64);
    stats.insert("total_values".to_string(), value_count as f64);
    
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 1234567890, vec![1.0, 2.0, 3.0]);
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = DataRecord::new(0, 1234567890, vec![1.0, 2.0]);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, 1234567890, vec![1.0, 2.0, 3.0]);
        transform_values(&mut record, 2.0);
        assert_eq!(record.values, vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            DataRecord::new(1, 1234567890, vec![1.0, 2.0]),
            DataRecord::new(2, 1234567891, vec![3.0, 4.0]),
        ];
        
        let stats = calculate_statistics(&records);
        assert_eq!(stats["total_records"], 2.0);
        assert_eq!(stats["average_value"], 2.5);
        assert_eq!(stats["total_values"], 4.0);
    }
}use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);

        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    fn save_filtered_to_csv(&self, category: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        let file = File::create(output_path)?;
        let mut wtr = WriterBuilder::new().has_headers(true).from_writer(file);

        for record in filtered {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }
}

fn process_data_sample() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.load_from_csv("input_data.csv")?;
    
    println!("Total records loaded: {}", processor.records.len());
    
    if let Some(avg) = processor.calculate_average() {
        println!("Average value: {:.2}", avg);
    }
    
    let filtered = processor.filter_by_category("premium");
    println!("Premium records: {}", filtered.len());
    
    if let Some(max_record) = processor.find_max_value() {
        println!("Max value record: {:?}", max_record);
    }
    
    processor.save_filtered_to_csv("premium", "premium_records.csv")?;
    
    Ok(())
}