
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    records: Vec<HashMap<String, f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if let Some(header_result) = lines.next() {
            let header_line = header_result?;
            let headers: Vec<String> = header_line
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();

            for line_result in lines {
                let line = line_result?;
                let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

                if values.len() == headers.len() {
                    let mut record = HashMap::new();
                    for (i, header) in headers.iter().enumerate() {
                        if let Ok(num) = values[i].parse::<f64>() {
                            record.insert(header.clone(), num);
                        }
                    }
                    self.records.push(record);
                }
            }
        }

        Ok(())
    }

    pub fn calculate_statistics(&self, column_name: &str) -> Option<(f64, f64, f64)> {
        let values: Vec<f64> = self.records
            .iter()
            .filter_map(|record| record.get(column_name).copied())
            .collect();

        if values.is_empty() {
            return None;
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;

        let variance: f64 = values.iter()
            .map(|value| {
                let diff = mean - value;
                diff * diff
            })
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        Some((mean, variance, std_dev))
    }

    pub fn filter_records(&self, column_name: &str, threshold: f64) -> Vec<HashMap<String, f64>> {
        self.records
            .iter()
            .filter(|record| {
                record.get(column_name)
                    .map(|&value| value > threshold)
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,score").unwrap();
        writeln!(temp_file, "1,23.5,0.8").unwrap();
        writeln!(temp_file, "2,17.2,0.6").unwrap();
        writeln!(temp_file, "3,31.7,0.9").unwrap();
        
        let result = processor.load_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 3);
        
        let stats = processor.calculate_statistics("value");
        assert!(stats.is_some());
        
        let filtered = processor.filter_records("score", 0.7);
        assert_eq!(filtered.len(), 2);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use serde::{Deserialize, Serialize};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

impl Record {
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        let mut count = 0;
        for result in csv_reader.deserialize() {
            let record: Record = result?;
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn save_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);

        for record in &self.records {
            csv_writer.serialize(record)?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    pub fn filter_by_value(&self, threshold: f64) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.value > threshold)
            .cloned()
            .collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn add_record(&mut self, record: Record) {
        if record.is_valid() {
            self.records.push(record);
        }
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
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 10.5,
            active: true,
        };
        assert!(valid_record.is_valid());

        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            value: -5.0,
            active: false,
        };
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();
        
        let record = Record {
            id: 1,
            name: "Sample".to_string(),
            value: 100.0,
            active: true,
        };
        
        processor.add_record(record);
        assert_eq!(processor.get_record_count(), 1);
        assert_eq!(processor.calculate_total(), 100.0);
        
        let filtered = processor.filter_by_value(50.0);
        assert_eq!(filtered.len(), 1);
        
        processor.clear();
        assert_eq!(processor.get_record_count(), 0);
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let mut processor = DataProcessor::new();
        
        let test_data = vec![
            Record {
                id: 1,
                name: "First".to_string(),
                value: 25.5,
                active: true,
            },
            Record {
                id: 2,
                name: "Second".to_string(),
                value: 75.0,
                active: false,
            },
        ];
        
        for record in test_data {
            processor.add_record(record);
        }
        
        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();
        
        processor.save_to_csv(path)?;
        
        let mut new_processor = DataProcessor::new();
        let loaded_count = new_processor.load_from_csv(path)?;
        
        assert_eq!(loaded_count, 2);
        assert_eq!(new_processor.get_record_count(), 2);
        
        Ok(())
    }
}