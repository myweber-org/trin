use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub timestamp: String,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, timestamp: String) -> Self {
        DataRecord {
            id,
            name,
            value,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.timestamp.is_empty()
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
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

            let timestamp = parts[3].to_string();

            let record = DataRecord::new(id, name, value, timestamp);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_by_value(&self, min_value: f64, max_value: f64) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.value >= min_value && r.value <= max_value)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_by_id(&self, target_id: u32) -> Option<&DataRecord> {
        self.records.iter().find(|r| r.id == target_id)
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, "Test".to_string(), 42.5, "2024-01-01".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, "".to_string(), -1.0, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,timestamp").unwrap();
        writeln!(temp_file, "1,Alice,100.5,2024-01-01").unwrap();
        writeln!(temp_file, "2,Bob,75.3,2024-01-02").unwrap();
        writeln!(temp_file, "3,Charlie,invalid,2024-01-03").unwrap();

        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 2);

        let filtered = processor.filter_by_value(70.0, 110.0);
        assert_eq!(filtered.len(), 2);

        let average = processor.calculate_average();
        assert!(average.is_some());
        assert!((average.unwrap() - 87.9).abs() < 0.1);

        let found = processor.find_by_id(1);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Alice");
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        DataRecord { id, value, category }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value >= 0.0 && !self.category.is_empty()
    }

    pub fn display(&self) -> String {
        format!("ID: {}, Value: {:.2}, Category: {}", self.id, self.value, self.category)
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { records: Vec::new() }
    }

    pub fn add_record(&mut self, record: DataRecord) {
        if record.is_valid() {
            self.records.push(record);
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if parts.len() == 3 {
                if let (Ok(id), Ok(value)) = (parts[0].parse::<u32>(), parts[1].parse::<f64>()) {
                    let record = DataRecord::new(id, value, parts[2].to_string());
                    if record.is_valid() {
                        self.records.push(record);
                        count += 1;
                    }
                }
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

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        let values: Vec<f64> = self.records.iter().map(|record| record.value).collect();
        
        if values.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = values.iter().sum::<f64>() / values.len() as f64;

        (min, max, avg)
    }

    pub fn count_records(&self) -> usize {
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
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 10.5, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(0, -5.0, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        processor.add_record(DataRecord::new(1, 10.0, "CategoryA".to_string()));
        processor.add_record(DataRecord::new(2, 20.0, "CategoryA".to_string()));
        processor.add_record(DataRecord::new(3, 30.0, "CategoryB".to_string()));

        assert_eq!(processor.count_records(), 3);
        
        let filtered = processor.filter_by_category("CategoryA");
        assert_eq!(filtered.len(), 2);

        let avg = processor.calculate_average();
        assert_eq!(avg, Some(20.0));

        let stats = processor.get_statistics();
        assert_eq!(stats, (10.0, 30.0, 20.0));
    }
}